//! The `scale` style: an element painted larger or smaller around its centre, with everything in it.
//! Layout and hit testing keep the element's real box, so a press that shrinks a button or a pop that
//! grows an icon moves nothing around it.

use std::panic::Location;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels,
    Window,
};

/// `element` painted at `scale` around its centre, or `element` as it is when there is no scale.
pub(super) fn scaled(element: AnyElement, scale: Option<f32>) -> AnyElement {
    match scale {
        Some(factor) if (factor - 1.0).abs() > f32::EPSILON => Scaled { child: element, factor }.into_any_element(),
        _ => element,
    }
}

struct Scaled {
    child: AnyElement,
    factor: f32,
}

impl Element for Scaled {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut (),
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        window.with_element_scale(bounds.center(), self.factor, |window| child.paint(window, cx));
    }
}

impl IntoElement for Scaled {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}
