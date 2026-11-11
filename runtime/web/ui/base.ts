// What a page needs once for the primitives to behave as the host's do: every element a flex
// container, positioned and border-box, the host's colours and fonts, and the layer overlays are
// drawn in. Installed by `start`, and by the first primitive that needs it.

import { SANS } from "./css";

/** Where `buildWeb` puts the host's fonts, relative to the page. */
const FONTS = "fonts";

const face = (family: string, file: string, weight: number): string =>
  `@font-face{font-family:"${family}";font-weight:${weight};font-style:normal;font-display:swap;` +
  `src:local("${family}"),url(${FONTS}/${file}) format("truetype")}`;

const BASE = `
${face("Geist", "Geist-400.ttf", 400)}
${face("Geist", "Geist-500.ttf", 500)}
${face("Geist", "Geist-600.ttf", 600)}
${face("JetBrains Mono", "JetBrainsMono-Regular.ttf", 400)}
${face("JetBrains Mono", "JetBrainsMono-Bold.ttf", 700)}
html,body{margin:0;height:100%;background:#18181b;color:#f4f4f5;font-family:${SANS};font-size:14px;line-height:1.4;-webkit-font-smoothing:antialiased;-webkit-text-size-adjust:100%}
body{position:relative;overflow:auto}
#gpui-root{position:absolute;top:0;left:0;right:0;bottom:0;display:flex;flex-direction:column}
.g-view,.g-button,.g-scroll,.g-overlay,.g-canvas,.g-icon,.g-image,.g-input,.g-text{box-sizing:border-box}
.g-view,.g-button,.g-scroll,.g-overlay,.g-canvas{position:relative;display:flex;flex-direction:column;flex-shrink:1}
.g-view:focus,.g-button:focus,.g-overlay:focus,.g-input:focus{outline:none}
.g-button{flex-direction:row;align-items:center;appearance:none;border:0;margin:0;padding:0;background:none;color:inherit;font:inherit;text-align:inherit;cursor:pointer;user-select:none}
.g-button[aria-disabled=true]{cursor:default}
.g-text{display:block;position:relative;white-space:normal}
.g-text .g-text{display:inline}
.g-text > :not(.g-text){display:inline-flex;vertical-align:baseline}
.g-scroll{overflow-x:hidden;overflow-y:auto;overflow-anchor:none;scrollbar-width:thin;scrollbar-color:#52525b transparent}
.g-scroll[data-horizontal]{flex-direction:row;overflow-x:auto;overflow-y:hidden}
.g-pointer{user-select:none;-webkit-user-select:none;touch-action:none}
.g-view,.g-canvas,.g-image,.g-icon{pointer-events:none}
.g-text,.g-button,.g-scroll,.g-input,.g-overlay,.g-listening{pointer-events:auto}
.g-canvas{overflow:hidden;flex-shrink:0}
.g-canvas > canvas{position:absolute;top:0;left:0;width:100%;height:100%;display:block}
.g-icon{display:inline-flex;position:relative;width:1em;height:1em;flex-shrink:0;align-items:center;justify-content:center}
.g-image{display:block;position:relative;max-width:none;flex-shrink:0}
.g-input{appearance:none;background:transparent;border:0;margin:0;padding:0;color:inherit;font:inherit;line-height:inherit;min-width:0;resize:none;display:block;position:relative;flex-shrink:1;text-align:inherit;cursor:text}
.g-input::placeholder{color:currentColor;opacity:.4}
textarea.g-input{overflow:auto;scrollbar-width:thin;scrollbar-color:#52525b transparent}
#gpui-layer{position:fixed;top:0;left:0;width:0;height:0;z-index:2147483000;pointer-events:none}
.g-overlay{position:fixed;top:0;left:0;pointer-events:auto;flex-shrink:0}
.g-overlay[data-passthrough],.g-overlay[data-passthrough] *{pointer-events:none!important}
.g-overlay:empty{display:none}
`;

let installed = false;

/** Adds the base stylesheet and stops the browser from opening a file that is dropped outside any element listening for it. */
export function installBase(): void {
  if (installed || typeof document === "undefined") {
    return;
  }
  installed = true;
  const style = document.createElement("style");
  style.id = "gpui-base";
  style.textContent = BASE;
  document.head.prepend(style);
  const holdsFiles = (event: DragEvent): boolean => Array.from(event.dataTransfer?.types ?? []).includes("Files");
  for (const type of ["dragover", "drop"]) {
    window.addEventListener(type, (event) => {
      if (holdsFiles(event as DragEvent) && !event.defaultPrevented) {
        event.preventDefault();
        (event as DragEvent).dataTransfer!.dropEffect = "none";
      }
    });
  }
}

/** The element the app mounts into. It fills the window, so a root `View` with `flex: 1` does too. */
export function rootElement(): HTMLElement {
  installBase();
  let root = document.getElementById("gpui-root");
  if (root === null) {
    root = document.createElement("div");
    root.id = "gpui-root";
    document.body.append(root);
  }
  return root;
}

/** The layer overlays are drawn in, above the app. */
export function layerElement(): HTMLElement {
  installBase();
  let layer = document.getElementById("gpui-layer");
  if (layer === null) {
    layer = document.createElement("div");
    layer.id = "gpui-layer";
    document.body.append(layer);
  }
  return layer;
}
