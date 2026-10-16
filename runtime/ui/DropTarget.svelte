<!--
  Somewhere a `Draggable` can be dropped. While one is held over it, `ondragenter`, `ondragover`
  and `ondragleave` report it, and `ondrop` takes it when released. `accept` says what it takes;
  a drag it does not accept passes over it to whatever is under. Targets can nest: the innermost
  one under the pointer hears it.

    <DropTarget accept={(item) => item.kind === "file"} ondrop={({ data }) => add(data)} over={{ background: "#8b5cf622" }}>
      …
    </DropTarget>

  `over` is laid over `style` while an accepted drag is on it, and `active` while any
  accepted drag is under way. The children can read both from their snippet argument.
-->
<script lang="ts" generics="T">
  import { onMount, type Snippet } from "svelte";
  import View from "./View.svelte";
  import { accepts, addTarget, dragging, type DragInfo } from "./dnd.svelte";
  import type { Layout } from "./events";
  import type { Style } from "./style";

  interface Props {
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

  let { accept, ondragenter, ondragover, ondragleave, ondrop, style, over: overStyle, active: activeStyle, children }: Props = $props();

  let bounds: Layout = { x: 0, y: 0, width: 0, height: 0 };
  let over = $state(false);

  onMount(() =>
    addTarget({
      get bounds() {
        return bounds;
      },
      get accept() {
        return accept as ((data: any) => boolean) | undefined;
      },
      enter(info) {
        over = true;
        ondragenter?.(info as DragInfo<T>);
      },
      leave() {
        over = false;
        ondragleave?.();
      },
      over(info) {
        ondragover?.(info as DragInfo<T>);
      },
      drop(info) {
        ondrop?.(info as DragInfo<T>);
      },
    }),
  );

  const held = $derived(accepts({ accept: accept as ((data: any) => boolean) | undefined }));
  const look: Style = $derived({ ...style, ...(held ? activeStyle : undefined), ...(over ? overStyle : undefined) });
</script>

<View style={look} onlayout={(layout) => (bounds = layout)}>
  {@render children?.({ over, dragging: held })}
</View>
