//! svelte-gpui: runs a compiled Svelte app in QuickJS and draws what it builds
//! with GPUI. The Svelte side keeps its own tree of primitives and sends only what
//! changes; this side keeps a copy, styles each primitive from the object the app
//! gave it, and paints it.
//!
//! The `svelte-gpui` binary runs a bundle from the command line; an app's own binary
//! can call [`run`] with [`Options`] instead.

mod assets;
mod canvas;
mod diagnostics;
mod embedded;
mod errors;
mod field;
mod images;
mod io;
mod launch;
mod render;
mod runtime;
mod sourcemap;
mod style;
mod tree;
mod watch;
mod window_control;
mod workbench;

pub use embedded::{Embedded, Metadata};
pub use io::{CommandError, Commands, Emitter, Output};
pub use launch::{Options, run};
pub use runtime::compile;
