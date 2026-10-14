// What Grove's main.ts does: boot the kernel, then mount the app. The fake bridge
// must exist before any Grove module is evaluated.
// The runtime first, for the globals Grove's code expects. Then the preload script,
// which builds window.workbench over the main process's socket and has to run before
// any Grove module reads it.
import { start } from "@neoworks/svelte-gpui";
import "@grove/src/preload/index";
import { bootKernel } from "@grove/src/renderer/src/kernel/boot";
import App from "@grove/src/renderer/src/App.svelte";

async function main(): Promise<void> {
  console.log("booting the kernel");
  try {
    await bootKernel();
    console.log("the kernel is up");
  } catch (error) {
    console.error("kernel failed to boot", error);
  }
  try {
    start(App);
    console.log("mounted");
  } catch (error) {
    console.error("mount failed", (error as Error).stack);
  }
}

void main();
