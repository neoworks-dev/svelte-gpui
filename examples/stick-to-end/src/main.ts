// The library must be imported before anything that pulls in Svelte's client runtime.
import { start } from "@neoworks/svelte-gpui";
import protocol from "../../../protocol.json";
import App from "./App.svelte";

// Counts what the app sends the host, so the figures the app logs can say how many scroll requests
// it took to settle: every one is a round trip the host's own anchoring does not need. A call to `scrollTo` sends
// two, one for each axis.
const scrollTo = protocol.operations.indexOf("scrollTo");
const host = (globalThis as any).host;
const apply = host.apply.bind(host);
const sent = { batches: 0, scrollRequests: 0 };
(globalThis as any).__sent = sent;
host.apply = (batch: string): void => {
  sent.batches++;
  for (const operation of JSON.parse(batch) as unknown[][]) {
    if (operation[0] === scrollTo) {
      sent.scrollRequests++;
    }
  }
  apply(batch);
};

start(App, {}, { window: { title: "Stick to end", width: 760, height: 640 } });
