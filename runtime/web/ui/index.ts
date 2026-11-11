// The primitives a svelte-gpui app is built from, for the browser: the same names, props and events as
// `runtime/ui/index.ts`, drawn as DOM elements. The components that are built from others (`List`,
// `Popover`, `Menu`, …) are the native target's own, since they only use the primitives, which
// `#primitives` swaps for these.

export { default as View } from "./View.svelte";
export { default as Text } from "./Text.svelte";
export { default as Button } from "./Button.svelte";
export { default as Input } from "./Input.svelte";
export { default as Scroll } from "./Scroll.svelte";
export { default as Image } from "./Image.svelte";
export { default as Icon } from "./Icon.svelte";
export { default as Presence } from "./Presence.svelte";
export { default as Canvas } from "./Canvas.svelte";
export { default as Overlay } from "./Overlay.svelte";

export { default as List } from "../../ui/List.svelte";
export { default as Tree } from "../../ui/Tree.svelte";
export { default as Popover } from "../../ui/Popover.svelte";
export { default as Tooltip } from "../../ui/Tooltip.svelte";
export { default as Menu } from "../../ui/Menu.svelte";
export { default as ContextMenu } from "../../ui/ContextMenu.svelte";
export { default as Select } from "../../ui/Select.svelte";
export { default as Draggable } from "../../ui/Draggable.svelte";
export { default as DropTarget } from "../../ui/DropTarget.svelte";
export { dragging } from "../../ui/dnd.svelte";
export type { DragInfo } from "../../ui/dnd.svelte";
export type { MenuItem } from "../../ui/menu";
export type { TreeRow } from "../../ui/tree";
export type { CloseReason, PopoverApi } from "../../ui/overlay-stack";
export * as overlayLook from "../../ui/overlay-look";
export { encode, merge } from "../../ui/style";
export type { Align, Color, Pen, Shadow, Sides, Size, Style, Transition } from "../../ui/style";
export type {
  DroppedFile,
  FileDragEvent,
  KeyEvent,
  Layout,
  Modifiers,
  OutsideEvent,
  PointerEvent,
  ScrollPosition,
  WheelEvent,
} from "../../ui/events";
