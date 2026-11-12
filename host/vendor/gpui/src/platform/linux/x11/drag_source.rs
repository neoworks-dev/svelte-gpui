//! The sending half of XDND (https://www.freedesktop.org/wiki/Specifications/XDND/): a drag
//! this client started, carried to whichever XdndAware window is under the pointer. The
//! pointer stays grabbed by the button that started it, so the moves and the release still
//! come to the window the drag began in, and are passed on here.

use std::sync::atomic::{AtomicU64, Ordering};

use futures::channel::oneshot;
use util::ResultExt;
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        self, AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, PropMode,
        SelectionNotifyEvent, SelectionRequestEvent,
    },
    wrapper::ConnectionExt as _,
    xcb_ffi::XCBConnection,
};

use super::{XcbAtoms, check_reply};
use crate::{ExternalDragOutcome, platform::linux::DraggedFiles};

/// The XDND version this side speaks.
const XDND_VERSION: u32 = 5;
/// How deep below the root a window that takes drops is looked for.
const MAXIMUM_DEPTH: usize = 16;

/// The window under the pointer that takes drops, and what it last said about this one.
struct Target {
    window: xproto::Window,
    version: u32,
    accepts: bool,
    action: xproto::Atom,
}

/// Whether a drag goes on after an event, or how it ended.
pub(crate) enum Progress {
    Continues,
    Ended(ExternalDragOutcome),
}

/// Numbers the drags, so a timer set for one cannot end the next.
static NEXT_DRAG_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) struct OutgoingXdnd {
    id: u64,
    /// The window the drag began in, which owns `XdndSelection` while it lasts.
    source: xproto::Window,
    files: DraggedFiles,
    target: Option<Target>,
    /// The pointer on the root window, and the server time of its last move.
    position: (i16, i16),
    time: xproto::Timestamp,
    /// A target answers each position with a status, and is sent no other until it has.
    awaiting_status: bool,
    position_pending: bool,
    /// The button was released before the target answered the last position.
    release_pending: bool,
    /// The drop was sent, and the target has yet to say it finished.
    dropped: bool,
    ended: Option<oneshot::Sender<ExternalDragOutcome>>,
}

impl OutgoingXdnd {
    /// Takes `XdndSelection` for `source`, which targets read the files from. `None` when the
    /// server refuses it.
    pub fn start(
        connection: &XCBConnection,
        atoms: &XcbAtoms,
        source: xproto::Window,
        files: DraggedFiles,
        ended: oneshot::Sender<ExternalDragOutcome>,
    ) -> Option<Self> {
        check_reply(
            || "Failed to take XdndSelection",
            connection.set_selection_owner(source, atoms.XdndSelection, x11rb::CURRENT_TIME),
        )
        .log_err()?;
        Some(OutgoingXdnd {
            id: NEXT_DRAG_ID.fetch_add(1, Ordering::Relaxed),
            source,
            files,
            target: None,
            position: (0, 0),
            time: x11rb::CURRENT_TIME,
            awaiting_status: false,
            position_pending: false,
            release_pending: false,
            dropped: false,
            ended: Some(ended),
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// Says how the drag ended to whoever started it.
    pub fn end(mut self, outcome: ExternalDragOutcome) {
        if let Some(ended) = self.ended.take() {
            ended.send(outcome).ok();
        }
    }

    /// The pointer moved to `x`, `y` on `root`: the window under it hears it entered, and is
    /// told where the pointer is; the one it left hears that.
    pub fn moved(
        &mut self,
        connection: &XCBConnection,
        atoms: &XcbAtoms,
        root: xproto::Window,
        position: (i16, i16),
        time: xproto::Timestamp,
    ) {
        if self.dropped || self.release_pending {
            return;
        }
        self.position = position;
        self.time = time;
        let under = window_taking_drops(connection, atoms, root, position);
        let current = self.target.as_ref().map(|target| target.window);
        if under.map(|(window, _)| window) != current {
            self.leave(connection, atoms);
            if let Some((window, version)) = under {
                self.enter(connection, atoms, window, version);
            }
        }
        if self.target.is_none() {
            return;
        }
        if self.awaiting_status {
            self.position_pending = true;
            return;
        }
        self.send_position(connection, atoms);
    }

    /// The button was released: the target takes the drop if it said it would.
    pub fn released(&mut self, connection: &XCBConnection, atoms: &XcbAtoms) -> Progress {
        if self.awaiting_status {
            self.release_pending = true;
            return Progress::Continues;
        }
        self.drop_or_cancel(connection, atoms)
    }

    /// Escape: the target hears the drag left, and nothing takes it.
    pub fn cancel(&mut self, connection: &XCBConnection, atoms: &XcbAtoms) {
        self.leave(connection, atoms);
    }

    /// An `XdndStatus` from `[target, flags, rectangle, rectangle, action]`.
    pub fn status(
        &mut self,
        connection: &XCBConnection,
        atoms: &XcbAtoms,
        data: [u32; 5],
    ) -> Progress {
        let Some(target) = self
            .target
            .as_mut()
            .filter(|target| target.window == data[0])
        else {
            return Progress::Continues;
        };
        target.accepts = data[1] & 1 == 1;
        target.action = data[4];
        self.awaiting_status = false;
        if self.release_pending {
            return self.drop_or_cancel(connection, atoms);
        }
        if self.position_pending {
            self.position_pending = false;
            self.send_position(connection, atoms);
        }
        Progress::Continues
    }

    /// An `XdndFinished` from `[target, flags, action]`: the target is done with the files.
    pub fn finished(&mut self, atoms: &XcbAtoms, data: [u32; 5]) -> Progress {
        let Some(target) = self
            .target
            .as_ref()
            .filter(|target| target.window == data[0])
        else {
            return Progress::Continues;
        };
        if !self.dropped {
            return Progress::Continues;
        }
        // Before version 5 the message carries no result; the last status stands for it.
        if target.version < 5 {
            return Progress::Ended(outcome_of(atoms, target.action));
        }
        if data[1] & 1 == 0 {
            return Progress::Ended(ExternalDragOutcome::Cancelled);
        }
        Progress::Ended(outcome_of(atoms, data[2]))
    }

    /// Hands a target that asked for `XdndSelection` the files in the form it asked for.
    pub fn answer(
        &self,
        connection: &XCBConnection,
        atoms: &XcbAtoms,
        request: &SelectionRequestEvent,
    ) {
        let mut property = request.property;
        if property == x11rb::NONE {
            property = request.target;
        }
        let written = self.write_selection(connection, atoms, request, property);
        if !written {
            property = x11rb::NONE;
        }
        let notify = SelectionNotifyEvent {
            response_type: xproto::SELECTION_NOTIFY_EVENT,
            sequence: 0,
            time: request.time,
            requestor: request.requestor,
            selection: request.selection,
            target: request.target,
            property,
        };
        check_reply(
            || "Failed to answer a request for XdndSelection",
            connection.send_event(false, request.requestor, EventMask::NO_EVENT, notify),
        )
        .log_err();
        connection.flush().log_err();
    }

    fn write_selection(
        &self,
        connection: &XCBConnection,
        atoms: &XcbAtoms,
        request: &SelectionRequestEvent,
        property: xproto::Atom,
    ) -> bool {
        let requestor = request.requestor;
        if request.target == atoms.TARGETS {
            let targets = offered_types(atoms);
            let mut all = vec![atoms.TARGETS];
            all.extend(targets);
            all.push(atoms.TEXT_PLAIN);
            all.push(atoms.STRING);
            return check_reply(
                || "Failed to list the drag's types",
                connection.change_property32(
                    PropMode::REPLACE,
                    requestor,
                    property,
                    AtomEnum::ATOM,
                    &all,
                ),
            )
            .log_err()
            .is_some();
        }
        let bytes = match request.target {
            target if target == atoms.TextUriList => &self.files.uri_list,
            target if is_text(atoms, target) => &self.files.text,
            _ => return false,
        };
        check_reply(
            || "Failed to hand over the dragged files",
            connection.change_property8(
                PropMode::REPLACE,
                requestor,
                property,
                request.target,
                bytes,
            ),
        )
        .log_err()
        .is_some()
    }

    fn drop_or_cancel(&mut self, connection: &XCBConnection, atoms: &XcbAtoms) -> Progress {
        self.release_pending = false;
        let accepted = self.target.as_ref().is_some_and(|target| target.accepts);
        if !accepted {
            self.leave(connection, atoms);
            return Progress::Ended(ExternalDragOutcome::Cancelled);
        }
        let target = self.target.as_ref().expect("checked above").window;
        send(
            connection,
            atoms.XdndDrop,
            target,
            [self.source, 0, self.time, 0, 0],
        );
        self.dropped = true;
        Progress::Continues
    }

    fn enter(
        &mut self,
        connection: &XCBConnection,
        atoms: &XcbAtoms,
        window: xproto::Window,
        version: u32,
    ) {
        let version = version.min(XDND_VERSION);
        let [first, second, third] = offered_types(atoms);
        send(
            connection,
            atoms.XdndEnter,
            window,
            [self.source, version << 24, first, second, third],
        );
        self.target = Some(Target {
            window,
            version,
            accepts: false,
            action: x11rb::NONE,
        });
        self.awaiting_status = false;
        self.position_pending = false;
    }

    fn leave(&mut self, connection: &XCBConnection, atoms: &XcbAtoms) {
        let Some(target) = self.target.take() else {
            return;
        };
        send(
            connection,
            atoms.XdndLeave,
            target.window,
            [self.source, 0, 0, 0, 0],
        );
        self.awaiting_status = false;
        self.position_pending = false;
    }

    fn send_position(&mut self, connection: &XCBConnection, atoms: &XcbAtoms) {
        let Some(target) = self.target.as_ref() else {
            return;
        };
        let (x, y) = self.position;
        let packed = ((x as u16 as u32) << 16) | (y as u16 as u32);
        let data = [self.source, 0, packed, self.time, atoms.XdndActionCopy];
        send(connection, atoms.XdndPosition, target.window, data);
        self.awaiting_status = true;
    }
}

/// The types an Enter announces: three fit in the message, so no `XdndTypeList` is needed.
fn offered_types(atoms: &XcbAtoms) -> [xproto::Atom; 3] {
    [atoms.TextUriList, atoms.UTF8_STRING, atoms.TEXT_PLAIN_UTF8]
}

fn is_text(atoms: &XcbAtoms, target: xproto::Atom) -> bool {
    target == atoms.UTF8_STRING
        || target == atoms.TEXT_PLAIN_UTF8
        || target == atoms.TEXT_PLAIN
        || target == atoms.STRING
        || target == atoms.TEXT
}

fn outcome_of(atoms: &XcbAtoms, action: xproto::Atom) -> ExternalDragOutcome {
    if action == atoms.XdndActionMove {
        return ExternalDragOutcome::Moved;
    }
    if action == atoms.XdndActionLink {
        return ExternalDragOutcome::Linked;
    }
    ExternalDragOutcome::Copied
}

fn send(connection: &XCBConnection, message: xproto::Atom, target: xproto::Window, data: [u32; 5]) {
    let event = ClientMessageEvent::new(32, target, message, data);
    check_reply(
        || "Failed to send an XDND message",
        connection.send_event(false, target, EventMask::NO_EVENT, event),
    )
    .log_err();
    connection.flush().log_err();
}

/// The innermost window under `position` that says it takes drops, with its XDND version.
/// The search goes down from the root, past window manager frames, which take none.
fn window_taking_drops(
    connection: &XCBConnection,
    atoms: &XcbAtoms,
    root: xproto::Window,
    position: (i16, i16),
) -> Option<(xproto::Window, u32)> {
    let mut parent = root;
    for _ in 0..MAXIMUM_DEPTH {
        let child = connection
            .translate_coordinates(root, parent, position.0, position.1)
            .ok()?
            .reply()
            .ok()?
            .child;
        if child == x11rb::NONE {
            return None;
        }
        if let Some(version) = xdnd_version(connection, atoms, child) {
            return Some((child, version));
        }
        parent = child;
    }
    None
}

fn xdnd_version(
    connection: &XCBConnection,
    atoms: &XcbAtoms,
    window: xproto::Window,
) -> Option<u32> {
    let reply = connection
        .get_property(false, window, atoms.XdndAware, AtomEnum::ATOM, 0, 1)
        .ok()?
        .reply()
        .ok()?;
    reply.value32()?.next()
}
