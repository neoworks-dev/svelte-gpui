use crate::{
    AnyElement, AnyEntity, AnyWeakEntity, App, Bounds, ContentMask, Context, Element, ElementId,
    Entity, EntityId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, PaintIndex,
    Pixels, PrepaintStateIndex, Render, Style, StyleRefinement, TextStyle, WeakEntity,
};
use crate::{
    AvailableSpace, Hitbox, Length, PaintOperation, ParentElement, Point, RelocationHook, Size,
    Styled, div, size,
};
use crate::{Empty, Window};
use anyhow::Result;
use collections::FxHashSet;
use refineable::Refineable;
use std::mem;
use std::rc::Rc;
use std::{any::TypeId, fmt, ops::Range};

struct AnyViewState {
    prepaint_range: Range<PrepaintStateIndex>,
    paint_range: Range<PaintIndex>,
    cache_key: ViewCacheKey,
    accessed_entities: FxHashSet<EntityId>,
}

/// What a relocatable view (see [`AnyView::cached_relocatable`]) keeps between frames. Everything
/// that has a position is kept as it was recorded, without the clip of what the view sits in, and is
/// moved and clipped again each time the view is drawn from it.
#[derive(Default)]
struct RelocatableState {
    /// Whether anything has been recorded.
    valid: bool,
    /// False when what was recorded cannot be drawn again somewhere else (tooltips, deferred draws, input handlers).
    reusable: bool,
    /// Set when the view found out it was drawn at the wrong size, so the next frame lays it out again.
    stale: bool,
    /// Where the view was when it was recorded, and its size.
    origin: Point<Pixels>,
    size: Size<Pixels>,
    /// How far from `origin` it was last drawn.
    applied: Point<Pixels>,
    /// Whether the pointer was over the view when it was recorded, so what it painted may show hover.
    hovered: bool,
    text_style: TextStyle,
    opacity: f32,
    prepaint_range: Range<PrepaintStateIndex>,
    paint_range: Range<PaintIndex>,
    accessed_entities: FxHashSet<EntityId>,
    hitboxes: Vec<Hitbox>,
    operations: Vec<PaintOperation>,
    hooks: Vec<RelocationHook>,
}

/// What `request_layout` decided for a view, which `prepaint` carries out.
#[doc(hidden)]
pub enum ViewLayout {
    /// An ordinary or cached view: the element when it was rendered for layout, else nothing.
    Plain(Option<AnyElement>),
    /// A relocatable view that changed: rendered into the outer layout, so it has its own natural size.
    Inline(AnyElement, FxHashSet<EntityId>),
    /// A relocatable view that did not change: a leaf of the size it had.
    Leaf,
}

/// How far a relocatable view may differ in size from what it recorded and still be drawn from it.
const SIZE_TOLERANCE: f32 = 0.01;

/// How far off a view drawn again at its old height may be before the next frame is laid out again.
const RELAYOUT_TOLERANCE: f32 = 0.5;

#[derive(Default)]
struct ViewCacheKey {
    bounds: Bounds<Pixels>,
    content_mask: ContentMask<Pixels>,
    text_style: TextStyle,
}

impl<V: Render> Element for Entity<V> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::View(self.entity_id()))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut element = self.update(cx, |view, cx| view.render(window, cx).into_any_element());
        let layout_id = window.with_rendered_view(self.entity_id(), |window| {
            element.request_layout(window, cx)
        });
        (layout_id, element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.set_view_id(self.entity_id());
        window.with_rendered_view(self.entity_id(), |window| element.prepaint(window, cx));
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_rendered_view(self.entity_id(), |window| element.paint(window, cx));
    }
}

/// A dynamically-typed handle to a view, which can be downcast to a [Entity] for a specific type.
#[derive(Clone, Debug)]
pub struct AnyView {
    entity: AnyEntity,
    render: fn(&AnyView, &mut Window, &mut App) -> AnyElement,
    cached_style: Option<Rc<StyleRefinement>>,
    relocatable: bool,
}

impl<V: Render> From<Entity<V>> for AnyView {
    fn from(value: Entity<V>) -> Self {
        AnyView {
            entity: value.into_any(),
            render: any_view::render::<V>,
            cached_style: None,
            relocatable: false,
        }
    }
}

impl AnyView {
    /// Like [`AnyView::cached`], for a view that is moved around, such as a row of a scrolling list,
    /// and whose size comes from its content.
    ///
    /// What the view painted is kept as it was, and drawn again somewhere else when only its position
    /// changed, moved by whole device pixels. Its layout is not computed again either: the layout
    /// around it gives it the height it had. A view that did change is laid out as part of its
    /// surroundings, so its new size takes effect in the same frame. It is drawn from scratch when its
    /// width changed (and the frame after is laid out again if that changed its height), while the
    /// pointer is over it, and when it made tooltips, deferred draws or input handlers.
    ///
    /// Unlike a view made with `cached`, this one does not force the views inside it to draw again when
    /// it does, and it must not hold views that are cached themselves. Elements that keep their own
    /// bounds ask to be told about a move with [`Window::on_relocation`].
    pub fn cached_relocatable(mut self, style: StyleRefinement) -> Self {
        self.cached_style = Some(style.into());
        self.relocatable = true;
        self
    }

    /// Indicate that this view should be cached when using it as an element.
    /// When using this method, the view's previous layout and paint will be recycled from the previous frame if [Context::notify] has not been called since it was rendered.
    /// The one exception is when [Window::refresh] is called, in which case caching is ignored.
    pub fn cached(mut self, style: StyleRefinement) -> Self {
        self.cached_style = Some(style.into());
        self
    }

    /// Convert this to a weak handle.
    pub fn downgrade(&self) -> AnyWeakView {
        AnyWeakView {
            entity: self.entity.downgrade(),
            render: self.render,
        }
    }

    /// Convert this to a [Entity] of a specific type.
    /// If this handle does not contain a view of the specified type, returns itself in an `Err` variant.
    pub fn downcast<T: 'static>(self) -> Result<Entity<T>, Self> {
        match self.entity.downcast() {
            Ok(entity) => Ok(entity),
            Err(entity) => Err(Self {
                entity,
                render: self.render,
                cached_style: self.cached_style,
                relocatable: self.relocatable,
            }),
        }
    }

    /// Gets the [TypeId] of the underlying view.
    pub fn entity_type(&self) -> TypeId {
        self.entity.entity_type
    }

    /// Gets the entity id of this handle.
    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id()
    }
}

impl PartialEq for AnyView {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl Eq for AnyView {}

impl AnyView {
    /// Whether this view is drawn from what it recorded, wherever it is.
    fn is_relocatable(&self, window: &Window, cx: &App) -> bool {
        self.relocatable && self.cached_style.is_some() && !window.is_inspector_picking(cx)
    }

    /// Lays out a relocatable view: as a leaf of the height it had when nothing changed, otherwise
    /// as part of the layout around it.
    fn request_relocatable_layout(
        &mut self,
        global_id: &GlobalElementId,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ViewLayout) {
        let entity_id = self.entity_id();
        let kept_height = window.with_element_state::<RelocatableState, _>(global_id, |state, window| {
            let state = state.unwrap_or_default();
            let unchanged = state.valid
                && state.reusable
                && !state.stale
                && !window.refreshing
                && !window.dirty_views.contains(&entity_id);
            let height = if unchanged { Some(state.size.height) } else { None };
            (height, state)
        });
        if let Some(height) = kept_height {
            let mut root_style = Style::default();
            root_style.refine(self.cached_style.as_ref().unwrap());
            if matches!(root_style.size.height, Length::Auto) {
                root_style.size.height = Length::Definite(height.into());
            }
            return (window.request_layout(root_style, None, cx), ViewLayout::Leaf);
        }
        let (rendered, accessed_entities) = cx.detect_accessed_entities(|cx| {
            let mut element = (self.render)(self, window, cx);
            let layout_id = element.request_layout(window, cx);
            (element, layout_id)
        });
        let (element, layout_id) = rendered;
        (layout_id, ViewLayout::Inline(element, accessed_entities))
    }

    /// Prepaints a relocatable view, recording what it makes unless it can be drawn from what it
    /// recorded before.
    fn prepaint_relocatable(
        &mut self,
        global_id: &GlobalElementId,
        bounds: Bounds<Pixels>,
        layout: ViewLayout,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        match layout {
            ViewLayout::Inline(mut element, accessed_entities) => {
                let recording = window.begin_prepaint_recording();
                element.prepaint(window, cx);
                let recorded = window.end_prepaint_recording(recording);
                Self::keep_recording(global_id, bounds, recorded, accessed_entities, window);
                Some(element)
            }
            ViewLayout::Leaf => self.prepaint_leaf(global_id, bounds, window, cx),
            ViewLayout::Plain(element) => element,
        }
    }

    fn keep_recording(
        global_id: &GlobalElementId,
        bounds: Bounds<Pixels>,
        recorded: crate::RecordedPrepaint,
        accessed_entities: FxHashSet<EntityId>,
        window: &mut Window,
    ) {
        window.with_element_state::<RelocatableState, _>(global_id, |state, window| {
            let mut state = state.unwrap_or_default();
            state.valid = true;
            state.reusable = recorded.reusable;
            state.stale = false;
            state.origin = bounds.origin;
            state.size = bounds.size;
            state.applied = Point::default();
            state.hovered = bounds.contains(&window.mouse_position());
            state.text_style = window.text_style();
            state.opacity = window.element_opacity();
            state.prepaint_range = recorded.range;
            state.paint_range = PaintIndex::default()..PaintIndex::default();
            state.accessed_entities = accessed_entities;
            state.hitboxes = recorded.hitboxes;
            state.operations = Vec::new();
            state.hooks = recorded.hooks;
            ((), state)
        });
    }

    /// A relocatable view that did not change: moves what it recorded to where the layout put it. A
    /// view that cannot be moved this way is rendered again, laid out at the width it was given.
    fn prepaint_leaf(
        &mut self,
        global_id: &GlobalElementId,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        let pointer = window.mouse_position();
        let fallback = window.with_element_state::<RelocatableState, _>(global_id, |state, window| {
            let mut state = state.unwrap_or_default();
            let same_size = (state.size.width.0 - bounds.size.width.0).abs() < SIZE_TOLERANCE
                && (state.size.height.0 - bounds.size.height.0).abs() < SIZE_TOLERANCE;
            // Hover is painted while recording, so a view the pointer is over, or was over when
            // it recorded, is drawn again.
            let touched = bounds.contains(&pointer) || state.hovered;
            // The text style and opacity of what the view sits in only exist while prepainting.
            let same_surroundings =
                state.text_style == window.text_style() && state.opacity == window.element_opacity();
            if !(state.valid && state.reusable && same_size && same_surroundings && !touched) {
                return (true, state);
            }
            let delta = window.snap_to_device_pixels(bounds.origin - state.origin);
            let start = window.prepaint_index();
            window.reuse_prepaint_relocated(state.prepaint_range.clone(), &state.hitboxes, delta);
            cx.entities.extend_accessed(&state.accessed_entities);
            state.prepaint_range = start..window.prepaint_index();
            let moved_by = delta - state.applied;
            for hook in &state.hooks {
                hook(moved_by);
            }
            state.applied = delta;
            (false, state)
        });
        if !fallback {
            return None;
        }
        Some(self.prepaint_rerendered(global_id, bounds, window, cx))
    }

    /// Renders the view again at the width the layout around it gave it, and records it. If that
    /// comes to another size than the layout allowed, the next frame is laid out again.
    fn prepaint_rerendered(
        &mut self,
        global_id: &GlobalElementId,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (rendered, accessed_entities) = cx.detect_accessed_entities(|cx| {
            let element = (self.render)(self, window, cx);
            let mut wrapper = div().flex().flex_col().w(bounds.size.width).child(element).into_any_element();
            let natural = wrapper.layout_as_root(
                size(AvailableSpace::Definite(bounds.size.width), AvailableSpace::MaxContent),
                window,
                cx,
            );
            (wrapper, natural)
        });
        let (mut wrapper, natural) = rendered;
        let recording = window.begin_prepaint_recording();
        wrapper.prepaint_at(bounds.origin, window, cx);
        let recorded = window.end_prepaint_recording(recording);
        Self::keep_recording(global_id, bounds, recorded, accessed_entities, window);
        // Only the height can have changed on its own: the width is the layout's to give, and was given.
        if (natural.height.0 - bounds.size.height.0).abs() >= RELAYOUT_TOLERANCE {
            window.with_element_state::<RelocatableState, _>(global_id, |state, _| {
                let mut state = state.unwrap_or_default();
                state.stale = true;
                ((), state)
            });
            window.request_animation_frame();
        }
        wrapper
    }

    fn paint_relocatable(
        &mut self,
        global_id: &GlobalElementId,
        element: &mut Option<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_element_state::<RelocatableState, _>(global_id, |state, window| {
            let mut state = state.unwrap_or_default();
            match element {
                Some(element) => {
                    let recording = window.begin_paint_recording();
                    element.paint(window, cx);
                    let recorded = window.end_paint_recording(recording);
                    state.paint_range = recorded.range;
                    state.operations = recorded.operations;
                    state.hooks.extend(recorded.hooks);
                    state.reusable = state.reusable && recorded.reusable;
                }
                None => {
                    let start = window.paint_index();
                    window.reuse_paint_relocated(
                        state.paint_range.clone(),
                        &state.operations,
                        state.applied,
                    );
                    state.paint_range = start..window.paint_index();
                }
            }
            ((), state)
        });
    }
}

impl Element for AnyView {
    type RequestLayoutState = ViewLayout;
    type PrepaintState = Option<AnyElement>;

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::View(self.entity_id()))
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        window.with_rendered_view(self.entity_id(), |window| {
            if self.is_relocatable(window, cx) {
                return self.request_relocatable_layout(global_id.unwrap(), window, cx);
            }
            // Disable caching when inspecting so that mouse_hit_test has all hitboxes.
            let caching_disabled = window.is_inspector_picking(cx);
            match self.cached_style.as_ref() {
                Some(style) if !caching_disabled => {
                    let mut root_style = Style::default();
                    root_style.refine(style);
                    let layout_id = window.request_layout(root_style, None, cx);
                    (layout_id, ViewLayout::Plain(None))
                }
                _ => {
                    let mut element = (self.render)(self, window, cx);
                    let layout_id = element.request_layout(window, cx);
                    (layout_id, ViewLayout::Plain(Some(element)))
                }
            }
        })
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        window.set_view_id(self.entity_id());
        let layout = mem::replace(layout, ViewLayout::Plain(None));
        window.with_rendered_view(self.entity_id(), |window| {
            if self.is_relocatable(window, cx) {
                return self.prepaint_relocatable(global_id.unwrap(), bounds, layout, window, cx);
            }
            let ViewLayout::Plain(element) = layout else {
                unreachable!("only a relocatable view lays out any other way");
            };
            if let Some(mut element) = element {
                element.prepaint(window, cx);
                return Some(element);
            }

            window.with_element_state::<AnyViewState, _>(
                global_id.unwrap(),
                |element_state, window| {
                    let content_mask = window.content_mask();
                    let text_style = window.text_style();

                    if let Some(mut element_state) = element_state
                        && element_state.cache_key.bounds == bounds
                        && element_state.cache_key.content_mask == content_mask
                        && element_state.cache_key.text_style == text_style
                        && !window.dirty_views.contains(&self.entity_id())
                        && !window.refreshing
                    {
                        let prepaint_start = window.prepaint_index();
                        window.reuse_prepaint(element_state.prepaint_range.clone());
                        cx.entities
                            .extend_accessed(&element_state.accessed_entities);
                        let prepaint_end = window.prepaint_index();
                        element_state.prepaint_range = prepaint_start..prepaint_end;

                        return (None, element_state);
                    }

                    let refreshing = mem::replace(&mut window.refreshing, true);
                    let prepaint_start = window.prepaint_index();
                    let (mut element, accessed_entities) = cx.detect_accessed_entities(|cx| {
                        let mut element = (self.render)(self, window, cx);
                        element.layout_as_root(bounds.size.into(), window, cx);
                        element.prepaint_at(bounds.origin, window, cx);
                        element
                    });

                    let prepaint_end = window.prepaint_index();
                    window.refreshing = refreshing;

                    (
                        Some(element),
                        AnyViewState {
                            accessed_entities,
                            prepaint_range: prepaint_start..prepaint_end,
                            paint_range: PaintIndex::default()..PaintIndex::default(),
                            cache_key: ViewCacheKey {
                                bounds,
                                content_mask,
                                text_style,
                            },
                        },
                    )
                },
            )
        })
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        element: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_rendered_view(self.entity_id(), |window| {
            if self.is_relocatable(window, cx) {
                self.paint_relocatable(global_id.unwrap(), element, window, cx);
                return;
            }
            let caching_disabled = window.is_inspector_picking(cx);
            if self.cached_style.is_some() && !caching_disabled {
                window.with_element_state::<AnyViewState, _>(
                    global_id.unwrap(),
                    |element_state, window| {
                        let mut element_state = element_state.unwrap();

                        let paint_start = window.paint_index();

                        if let Some(element) = element {
                            let refreshing = mem::replace(&mut window.refreshing, true);
                            element.paint(window, cx);
                            window.refreshing = refreshing;
                        } else {
                            window.reuse_paint(element_state.paint_range.clone());
                        }

                        let paint_end = window.paint_index();
                        element_state.paint_range = paint_start..paint_end;

                        ((), element_state)
                    },
                )
            } else {
                element.as_mut().unwrap().paint(window, cx);
            }
        });
    }
}

impl<V: 'static + Render> IntoElement for Entity<V> {
    type Element = Entity<V>;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl IntoElement for AnyView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// A weak, dynamically-typed view handle that does not prevent the view from being released.
pub struct AnyWeakView {
    entity: AnyWeakEntity,
    render: fn(&AnyView, &mut Window, &mut App) -> AnyElement,
}

impl AnyWeakView {
    /// Convert to a strongly-typed handle if the referenced view has not yet been released.
    pub fn upgrade(&self) -> Option<AnyView> {
        let entity = self.entity.upgrade()?;
        Some(AnyView {
            entity,
            render: self.render,
            cached_style: None,
            relocatable: false,
        })
    }
}

impl<V: 'static + Render> From<WeakEntity<V>> for AnyWeakView {
    fn from(view: WeakEntity<V>) -> Self {
        AnyWeakView {
            entity: view.into(),
            render: any_view::render::<V>,
        }
    }
}

impl PartialEq for AnyWeakView {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl std::fmt::Debug for AnyWeakView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyWeakView")
            .field("entity_id", &self.entity.entity_id)
            .finish_non_exhaustive()
    }
}

mod any_view {
    use crate::{AnyElement, AnyView, App, IntoElement, Render, Window};

    pub(crate) fn render<V: 'static + Render>(
        view: &AnyView,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let view = view.clone().downcast::<V>().unwrap();
        view.update(cx, |view, cx| view.render(window, cx).into_any_element())
    }
}

/// A view that renders nothing
pub struct EmptyView;

impl Render for EmptyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}
