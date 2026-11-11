# A stand-in for the desktop's file chooser portal (org.freedesktop.portal.FileChooser), for
# tests/web/serve.test.ts: it answers the headless host's dialogs on a private session bus, so nothing
# is shown on anybody's desktop. Every request's options are appended to MOCK_PORTAL_LOG as one JSON line.
# What it answers is under MOCK_PORTAL_ROOT: an open picks a.txt (and b.txt when `multiple`), or `folder` for a
# directory; a save picks the suggested name. A request whose accept label is "Cancel" is cancelled.
import json
import os

import dbus
import dbus.lowlevel
import dbus.mainloop.glib
import dbus.service
from gi.repository import GLib

LOG = os.environ["MOCK_PORTAL_LOG"]
ROOT = os.environ["MOCK_PORTAL_ROOT"]

dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()
name = dbus.service.BusName("org.freedesktop.portal.Desktop", bus)


def plain(value):
    if isinstance(value, (dbus.Array, list)):
        return [plain(v) for v in value]
    if isinstance(value, (dbus.Dictionary, dict)):
        return {str(k): plain(v) for k, v in value.items()}
    if isinstance(value, dbus.Struct):
        return [plain(v) for v in value]
    if isinstance(value, dbus.Boolean):
        return bool(value)
    if isinstance(value, (dbus.String, dbus.ObjectPath)):
        return str(value)
    if isinstance(value, (dbus.Int32, dbus.UInt32, dbus.Int64, dbus.UInt64, dbus.Byte)):
        return int(value)
    return value


class Chooser(dbus.service.Object):
    def __init__(self):
        super().__init__(bus, "/org/freedesktop/portal/desktop")

    def answer(self, kind, title, options, sender):
        options = plain(options)
        with open(LOG, "a") as log:
            log.write(json.dumps({"kind": kind, "title": str(title), "options": options}) + "\n")
        path = "/org/freedesktop/portal/desktop/request/%s/%s" % (sender[1:].replace(".", "_"), options["handle_token"])
        if options.get("accept_label") == "Cancel":
            code, uris = 1, []
        elif kind == "save":
            code, uris = 0, ["file://%s/%s" % (ROOT, options.get("current_name", "saved.txt"))]
        elif options.get("directory"):
            code, uris = 0, ["file://%s/folder" % ROOT]
        elif options.get("multiple"):
            code, uris = 0, ["file://%s/a.txt" % ROOT, "file://%s/b.txt" % ROOT]
        else:
            code, uris = 0, ["file://%s/a.txt" % ROOT]

        def respond():
            message = dbus.lowlevel.SignalMessage(path, "org.freedesktop.portal.Request", "Response")
            results = dbus.Dictionary({"uris": dbus.Array(uris, signature="s")} if uris else {}, signature="sv")
            message.append(dbus.UInt32(code), results, signature="ua{sv}")
            bus.send_message(message)
            return False

        GLib.timeout_add(100, respond)
        return dbus.ObjectPath(path)

    @dbus.service.method("org.freedesktop.portal.FileChooser", in_signature="ssa{sv}", out_signature="o", sender_keyword="sender")
    def OpenFile(self, parent, title, options, sender=None):
        return self.answer("open", title, options, sender)

    @dbus.service.method("org.freedesktop.portal.FileChooser", in_signature="ssa{sv}", out_signature="o", sender_keyword="sender")
    def SaveFile(self, parent, title, options, sender=None):
        return self.answer("save", title, options, sender)

    @dbus.service.method(dbus.PROPERTIES_IFACE, in_signature="ss", out_signature="v")
    def Get(self, interface, prop):
        return dbus.UInt32(3)

    @dbus.service.method(dbus.PROPERTIES_IFACE, in_signature="s", out_signature="a{sv}")
    def GetAll(self, interface):
        return {"version": dbus.UInt32(3)}


Chooser()
print("ready", flush=True)
GLib.MainLoop().run()
