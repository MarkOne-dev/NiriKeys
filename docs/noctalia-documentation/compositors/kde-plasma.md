# KDE Plasma

Noctalia can run as a shell on KDE Plasma (KWin), but Plasma is not
a first-class compositor integration the way Niri or Hyprland are.
Layer-shell bars and panels work; several features depend on
wlroots-style protocols that stock KWin does not expose, or on
Plasma-specific bridges that are less complete than the protocol
path.

This page lists the main gaps so you know what to expect before
switching.

## Autostart

Start Noctalia from Plasma's session autostart
(**System Settings → Autostart**), or add it to your login script:

```bash
noctalia
```

Wire launcher / control center / settings the same way as on other
compositors, using `noctalia msg ...` binds in Plasma's Shortcuts
settings or a custom script. See *Running Noctalia and IPC*.

## What Generally Works

- Layer-shell bars, panels, dock, OSD, notifications, launcher,
  control center, settings
- Workspaces via Plasma's virtual-desktop protocol
  (`org_kde_plasma_virtual_desktop`)
- Taskbar / dock / active window through a KWin scripting D-Bus
  bridge (not wlr/ext foreign-toplevel)
- System tray (Plasma StatusNotifier path)
- Notifications via Plasma's notification watcher (Noctalia does
  not own `org.freedesktop.Notifications` on Plasma)
- Theming of KDE apps via KColorScheme — see *GTK / Qt / KDE
  theming*

## Limitations

### Clipboard History

Clipboard history needs `ext-data-control-v1` or
`zwlr-data-control-unstable-v1`. Stock KWin usually does not
advertise either, so the clipboard panel and history widget stay
unavailable. Basic copy/paste in apps is unaffected.

### Clipboard Auto-Paste

Selecting a history entry can auto-send a paste key chord
(`Ctrl+V` / `Ctrl+Shift+V`, etc.). That path requires
`zwp_virtual_keyboard_v1`, which stock KWin typically does not
expose, so auto-paste is unavailable on Plasma. Installing `wtype`
does not change this because it requires the same protocol.
Selecting an entry still copies it when clipboard history is
available.

### Screenshots

The screenshot bar widget and `screenshot-region` /
`screenshot-fullscreen` IPC need `zwlr-screencopy-unstable-v1`.
That protocol is usually absent on KWin, so the widget is omitted
and IPC reports capture unavailable. Use Plasma's own screenshot
tools instead.

### Lock Screen Desktop Capture

`[lockscreen]` desktop capture (blurred live desktop behind the
lock) also needs screencopy. Without it, the lock screen uses the
wallpaper / configured image only. See *Shell → Lock screen*.

### Night Light

Noctalia night light uses `zwlr-gamma-control-unstable-v1`. Stock
KWin does not provide it, so Noctalia's night light stays disabled.
Use Plasma's Night Color instead. See *Night Light*.

### Compositor Blur and Better Blur

Noctalia publishes blur regions through `ext-background-effect-v1`
for the bar, dock, and notification surfaces. Stock KWin often
applies little or no useful blur for those surfaces.

Third-party Better Blur (and similar KWin effects) can make blur
visible, but they are not first-class Noctalia integrations. A
common issue: after bar or dock auto-hide, a leftover blurred
"halo" or ghost of the surface remains on screen even though
Noctalia clears the blur region when hiding. That leftover comes
from how the effect composites layer surfaces, not from Noctalia
failing to request a clear. Workarounds:

- Disable Better Blur for Noctalia / layer-shell windows in the
  effect's rules, or
- Avoid auto-hide while relying on Better Blur, or
- Prefer compositors with native `ext-background-effect` support
  (see *Niri blur* / *Hyprland*)

### Window Tracking (Taskbar, Dock, Screen Time)

KWin does not expose `wlr-foreign-toplevel-management` /
`ext-foreign-toplevel-list` to Noctalia. Window list, focus,
activate, and close go through an injected KWin script over D-Bus.
That works for many taskbar/dock cases, but it is more fragile than
the protocol path (script install, window identity matching, close
sometimes falling back to minimize).

Screen Time in the control center currently keys off the wlroots
foreign-toplevel active window path, so tracking is typically empty
or incomplete on Plasma even when the taskbar shows windows.

### Keyboard Layout Widget

There is no Plasma keyboard-layout compositor backend. The bar
keyboard-layout widget has nothing to drive for layout
detection/cycling unless you configure a custom cycle command. See
*Keyboard layout widget*.

### Overview / Wallpaper Backdrop

Overview-aware smart auto-hide and the Niri-style wallpaper
overview backdrop are compositor-specific. They are not available
on Plasma. See *Wallpaper*.

### Session Logout

Session exit on Plasma falls through to a generic
`loginctl terminate` path rather than Plasma's native logout /
`ksmserver` flow. Prefer Plasma's own Leave / Logout UI when you
need a full session shutdown.

### Tray Icons Disappearing

If you run Plasma apps (for example Dolphin) under another
compositor while KDE packages are installed, `kded6` can steal the
StatusNotifier watcher. See the FAQ tray note.

## Summary

| Feature | Plasma / KWin |
| --- | --- |
| Bars, panels, dock, launcher, CC | ✅ Works (layer-shell) |
| System tray | ✅ Works (Plasma StatusNotifier path) |
| Workspaces | ✅ Works (Plasma virtual desktops) |
| Taskbar / dock windows | ⚠️ Works via KWin script (degraded vs protocol) |
| Clipboard history | ❌ Usually unavailable (no data-control) |
| Clipboard auto-paste | ❌ Usually unavailable (no virtual-keyboard) |
| Screenshots (Noctalia) | ❌ Usually unavailable (no screencopy) |
| Lock desktop capture | ❌ Unavailable without screencopy |
| Night light (Noctalia) | ❌ Unavailable (no gamma-control) |
| Compositor blur | ⚠️ Weak stock; Better Blur can leave hide artifacts |
| Screen Time | ❌ Typically broken / empty |
| Keyboard layout widget | ❌ No backend |
| Overview backdrop | ❌ Niri-only |
