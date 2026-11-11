// The props of every component, in one place, for both targets: the native components in this
// directory and the DOM ones in `runtime/web/ui` take the same types, so they cannot drift.

import type { Snippet } from "svelte";
import type { TransitionConfig } from "svelte/transition";
import type { DragInfo } from "./dnd.svelte";
import type {
  FileDragEvent,
  KeyEvent,
  Layout,
  OutsideEvent,
  PointerEvent,
  ScrollPosition,
  WheelEvent,
} from "./events";
import type { MenuItem } from "./menu";
import type { CloseReason, PopoverApi } from "./overlay-stack";
import type { Color, Pen, Style } from "./style";
import type { TreeRow } from "./tree";

export interface ViewProps {
  style?: Style;
  hover?: Style;
  active?: Style;
  focus?: Style;
  /** Takes keyboard focus when pressed. */
  focusable?: boolean;
  /** A press here does not take focus from the field that has it, nor give it to anything. */
  keepFocus?: boolean;
  onpress?: (event: PointerEvent) => void;
  onhover?: (hovered: boolean) => void;
  /** A button went down here; `onmove` and `onup` follow wherever the pointer goes until release. */
  ondown?: (event: PointerEvent) => void;
  /** The buttons `ondown` takes. A press of another button passes to what is around the view. The default is all three. */
  buttons?: ("left" | "middle" | "right")[];
  onmove?: (event: PointerEvent) => void;
  onup?: (event: PointerEvent) => void;
  onwheel?: (event: WheelEvent) => void;
  onlayout?: (layout: Layout) => void;
  /** Like `onlayout`, but only when its width or height changed, not when it merely moved. */
  onresize?: (layout: Layout) => void;
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
  /** Files dragged in from another program entered, left, moved over this, or were dropped on it. */
  ondragenter?: (event: FileDragEvent) => void;
  ondragleave?: (event: FileDragEvent) => void;
  ondragover?: (event: FileDragEvent) => void;
  ondrop?: (event: FileDragEvent) => void;
  children?: Snippet;
}

export interface TextProps {
  style?: Style;
  /** Keep whitespace and line breaks as written. */
  pre?: boolean;
  onlayout?: (layout: Layout) => void;
  children?: Snippet;
}

export interface ButtonProps {
  style?: Style;
  hover?: Style;
  active?: Style;
  focus?: Style;
  /** Laid over `style` while the button is disabled. */
  disabledStyle?: Style;
  disabled?: boolean;
  onpress?: (event: PointerEvent) => void;
  onhover?: (hovered: boolean) => void;
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
  children?: Snippet;
}

export interface InputProps {
  value?: string;
  placeholder?: string;
  /** Several lines; Enter breaks the line instead of submitting. */
  multiline?: boolean;
  /** How many lines a multiline field shows at least. */
  rows?: number;
  /** Shows dots instead of the text. */
  secure?: boolean;
  /** Takes only digits, signs, a point and exponents. */
  numeric?: boolean;
  maxLength?: number;
  disabled?: boolean;
  readonly?: boolean;
  style?: Style;
  hover?: Style;
  focus?: Style;
  /** Every change to the text. */
  oninput?: (value: string) => void;
  /** Enter in a single-line field. */
  onsubmit?: (value: string) => void;
  /** The text changed since the field took focus, and focus left or Enter was pressed. */
  oncommit?: (value: string) => void;
  /** The cursor or the selection moved, with the text offsets (UTF-16, as for `select()`) of its two ends. */
  onselect?: (selection: { start: number; end: number }) => void;
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
}

export interface ScrollProps {
  style?: Style;
  horizontal?: boolean;
  /** What the host keeps in view as the content changes: the window's content, or that and the end. Vertical only. */
  anchor?: "content" | "end";
  onscroll?: (position: ScrollPosition) => void;
  onlayout?: (layout: Layout) => void;
  children?: Snippet;
}

export interface ImageProps {
  src: string;
  /** How the image fills its box. */
  fit?: "contain" | "cover" | "fill";
  style?: Style;
  onload?: () => void;
  onerror?: () => void;
}

export interface IconProps {
  svg?: string;
  src?: string;
  /** Width and height in pixels; the font size around it otherwise. */
  size?: number;
  color?: Color;
  style?: Style;
}

export interface CanvasProps {
  style?: Style;
  /** Keeps shapes and paints them as paths instead of pixels. Set it when the canvas is made. */
  vector?: boolean;
  ondown?: (event: PointerEvent) => void;
  onmove?: (event: PointerEvent) => void;
  onup?: (event: PointerEvent) => void;
  onhover?: (hovered: boolean) => void;
  onwheel?: (event: WheelEvent) => void;
  onlayout?: (layout: Layout) => void;
}

/** The methods of a `Canvas`, as `bind:this` gives them. */
export interface CanvasMethods {
  clear(color?: Color): void;
  begin(x: number, y: number, pen?: Pen): void;
  lineTo(x: number, y: number): void;
  end(): void;
  line(x0: number, y0: number, x1: number, y1: number, pen?: Pen): void;
  fillRect(x: number, y: number, width: number, height: number, color: Color): void;
  fillCircle(x: number, y: number, radius: number, color: Color): void;
}

export type TransitionFunction = (
  node: any,
  params: any,
  options: { direction: "in" | "out" | "both" },
) => TransitionConfig | (() => TransitionConfig);

/** A transition, or one with its parameters: `fade`, `[fly, { x: -20, duration: 300 }]`. */
export type TransitionProp = TransitionFunction | [TransitionFunction, unknown];

export interface PresenceProps {
  in?: TransitionProp;
  out?: TransitionProp;
  style?: Style;
  children?: Snippet;
}

export interface OverlayProps {
  style?: Style;
  hover?: Style;
  active?: Style;
  focus?: Style;
  /** Which edge of the parent it sits against. The default is `"bottom"`. */
  side?: "top" | "bottom" | "left" | "right";
  /** Where along that edge it lines up. The default is `"start"`. */
  align?: "start" | "center" | "end";
  /** The gap between the parent and the overlay, along `side`. */
  offset?: number;
  /** Slides it along the edge, toward the end. */
  crossOffset?: number;
  /** A point in the window to place it against, `[x, y]`, instead of the parent's box. */
  at?: [number, number];
  /** Moves to the opposite side when the chosen one has too little room. The default is on. */
  flip?: boolean;
  /** Slides back inside the window when it would stick out. The default is on. */
  shift?: boolean;
  /** The least distance kept from the window's edges. The default is 4. */
  margin?: number;
  /** At least as wide as the parent. */
  matchWidth?: boolean;
  /** Lets the pointer through to what is behind it, instead of blocking it. */
  passthrough?: boolean;
  /** Overlays with a higher layer are drawn over those with a lower one. */
  layer?: number;
  /** Takes keyboard focus when pressed, and when `focus()` is called. */
  focusable?: boolean;
  /** A press here does not take focus from the field that has it, nor give it to anything. */
  keepFocus?: boolean;
  /** A button went down outside it (and outside any overlay opened from inside it). */
  onoutside?: (event: OutsideEvent) => void;
  onpress?: (event: PointerEvent) => void;
  onhover?: (hovered: boolean) => void;
  onlayout?: (layout: Layout) => void;
  onkey?: (event: KeyEvent) => void;
  onfocus?: () => void;
  onblur?: () => void;
  children?: Snippet;
}

export interface ListProps<T> {
  items: T[];
  /** The height of every row, in pixels. Leave it out for rows of different heights. */
  rowHeight?: number;
  /** What a row not measured yet counts as, before any has been. The default is 40. */
  estimatedRowHeight?: number;
  row: Snippet<[T, number]>;
  /** Keeps a row's state, and its measured height, with its item as the list changes. */
  key?: (item: T, index: number) => unknown;
  /** How many rows past each edge of the window are built anyway, within a window's height of it. The default is 8. */
  overscan?: number;
  /**
   * Opens at the end, and stays there as items are added and rows change height, as a chat does, for as long
   * as the list is scrolled to its end. Scrolling up lets go of it; scrolling back to the end takes it again.
   */
  stickToEnd?: boolean;
  style?: Style;
  onscroll?: (position: ScrollPosition) => void;
}

export interface TreeProps<T> {
  /** The top level. */
  items: T[];
  /** Identifies a node, for what is expanded and selected. It must be unique and stay the same for the same node. */
  key: (node: T) => string | number;
  /** A node's children. Return nothing for a leaf. It is called for expanded branches only. */
  childrenOf?: (node: T) => T[] | undefined | null;
  /** Whether a node can be expanded. The default is that it has children, which a lazily loaded branch does not yet. */
  isBranch?: (node: T) => boolean;
  row: Snippet<[T, TreeRow]>;
  /** The keys of the nodes that are expanded. Replace it to change it; it is never changed in place. */
  expanded?: Set<string | number>;
  /** The key of the selected node. */
  selected?: string | number | undefined;
  /** The height of every row, in pixels, or `"auto"` to measure rows that differ. The default is 28. */
  rowHeight?: number | "auto";
  /** The indent of each level, in pixels. The default is 16. */
  indent?: number;
  overscan?: number;
  /** A node was selected, by the pointer or the keys. */
  onselect?: (node: T) => void;
  /** Enter, or a second press, on a node. */
  onactivate?: (node: T) => void;
  /** A branch was expanded: load its children here. */
  onexpand?: (node: T) => void;
  oncollapse?: (node: T) => void;
  /** The tree's style. Give it a bounded height. */
  style?: Style;
  /** Laid over every row's style. */
  rowStyle?: Style;
  /** Laid over a selected row's style. */
  selectedStyle?: Style;
}

export interface PopoverProps {
  open?: boolean;
  trigger?: Snippet<[PopoverApi]>;
  children?: Snippet<[PopoverApi]>;
  side?: "top" | "bottom" | "left" | "right";
  align?: "start" | "center" | "end";
  /** The gap between the trigger and the popover. The default is 6. */
  offset?: number;
  crossOffset?: number;
  flip?: boolean;
  shift?: boolean;
  margin?: number;
  /** At least as wide as the trigger. */
  matchWidth?: boolean;
  /** A point in the window to place it against, instead of the trigger. */
  at?: [number, number];
  /** The panel's look. The default is a dark card; pass `{}` for none. */
  style?: Style;
  /** The wrapper around the trigger, whose box the popover is placed against. */
  anchorStyle?: Style;
  closeOnOutside?: boolean;
  closeOnEscape?: boolean;
  /** Moves focus to the panel when it opens. Turn it off when the content takes focus itself. */
  focusPanel?: boolean;
  /** Gives focus back to what had it when it closes by Escape or a choice. The default is on. */
  restoreFocus?: boolean;
  layer?: number;
  onclose?: (reason: CloseReason) => void;
  onkey?: (event: KeyEvent) => void;
}

export interface TooltipProps {
  text?: string;
  content?: Snippet;
  /** Milliseconds the pointer rests before it shows. The default is 500. */
  delay?: number;
  side?: "top" | "bottom" | "left" | "right";
  align?: "start" | "center" | "end";
  /** The gap between the content and the tooltip. The default is 6. */
  offset?: number;
  /** Never shows. */
  disabled?: boolean;
  /** The tooltip's style. */
  style?: Style;
  /** The wrapper around the content, which is what the pointer rests on. */
  anchorStyle?: Style;
  children?: Snippet;
}

export interface MenuProps {
  items: MenuItem[];
  /** Called with the item and its index after the item's own `onselect`. */
  onselect?: (item: MenuItem, index: number) => void;
  /** Called after any choice. Close the popover here. */
  onclose?: () => void;
  /** The item highlighted when it appears. The default is the first one that is not disabled. */
  initial?: number;
  /** Takes keyboard focus when it appears. The default is on. */
  autofocus?: boolean;
  /** Scrolls when the rows are taller than this. */
  maxHeight?: number;
  /** The width, in pixels. The default is as wide as the widest row, and at least the popover's. */
  width?: number;
  style?: Style;
  /** Laid over every row's own style. */
  itemStyle?: Style;
}

export interface ContextMenuProps {
  /** The menu's items. Optional when `openAt` is always given its own. */
  items?: MenuItem[] | ((event: PointerEvent) => MenuItem[]);
  onselect?: (item: MenuItem, index: number) => void;
  onopen?: (event: PointerEvent) => void;
  /** The region's style. */
  style?: Style;
  /** The menu's own style. */
  menuStyle?: Style;
  /** The popover's panel style. */
  panelStyle?: Style;
  children?: Snippet;
}

export interface SelectOption<T> {
  value: T;
  label: string;
  disabled?: boolean;
}

export interface SelectProps<T> {
  /** The chosen option's value. */
  value?: T;
  /** Plain values, or `{ value, label, disabled }`. */
  options: (T | SelectOption<T>)[];
  placeholder?: string;
  disabled?: boolean;
  /** The tallest the list gets, in pixels, before it scrolls. The default is 260. */
  maxHeight?: number;
  onchange?: (value: T) => void;
  /** The button's style. */
  style?: Style;
  hover?: Style;
  focus?: Style;
  /** The list's panel style. */
  panelStyle?: Style;
}

export interface DraggableProps<T> {
  /** What a target is given when this is dropped on it. */
  data: T;
  disabled?: boolean;
  /** How far the pointer moves, in pixels, before the drag starts. The default is 4. */
  threshold?: number;
  style?: Style;
  hover?: Style;
  /** Laid over `style` on the original while it is being dragged. The default dims it. */
  draggingStyle?: Style;
  /** What follows the pointer. The default is a copy of the children. */
  preview?: Snippet<[T]>;
  /** Laid over the preview's wrapper, which is as wide as the original. */
  previewStyle?: Style;
  onstart?: (data: T) => void;
  /** The drag ended, dropped on a target or not. */
  onend?: (result: { data: T; dropped: boolean }) => void;
  children?: Snippet;
}

export interface DropTargetProps<T> {
  /** Which dragged data it takes. The default is everything. */
  accept?: (data: T) => boolean;
  ondragenter?: (info: DragInfo<T>) => void;
  ondragover?: (info: DragInfo<T>) => void;
  ondragleave?: () => void;
  ondrop?: (info: DragInfo<T>) => void;
  style?: Style;
  /** Laid over `style` while an accepted drag is over it. */
  over?: Style;
  /** Laid over `style` while any accepted drag is in progress, over it or not. */
  active?: Style;
  children?: Snippet<[{ over: boolean; dragging: boolean }]>;
}
