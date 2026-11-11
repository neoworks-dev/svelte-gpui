// Fails the type check when the browser target stops matching the native one: a name one exports and the other
// does not, or a type that differs. It runs nothing.

import type * as NativeRoot from "../index";
import type * as NativeUi from "../ui/index";
import type * as WebRoot from "./index";
import type * as WebUi from "./ui/index";

declare const nativeRoot: typeof NativeRoot;
declare const nativeUi: typeof NativeUi;
declare const webRoot: typeof WebRoot;
declare const webUi: typeof WebUi;

export const nativeRootFitsWeb: typeof WebRoot = nativeRoot;
export const webRootFitsNative: typeof NativeRoot = webRoot;
export const nativeUiFitsWeb: typeof WebUi = nativeUi;
export const webUiFitsNative: typeof NativeUi = webUi;
