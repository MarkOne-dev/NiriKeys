# IPC & Keybinds

IPC commands let you control Noctalia from a terminal, a compositor
keybind, a script, or a hook.

All commands use this shape:

```bash
noctalia msg <command>
```

For example:

```bash
noctalia msg dock-toggle
noctalia msg volume-up
noctalia msg panel-toggle launcher
noctalia msg screenshot-region
```

> **Note:** These pages list commands exactly as you run them. The
> same `noctalia msg ...` form is used in terminals, compositor
> keybinds, hooks, hot-corner commands, and idle command fields.

## Command Lists

### Shell

Status, config reload, settings, window switcher, and session
actions.

### Surfaces

Bar, panels, dock, desktop widgets, and lockscreen widgets.

### Media & UI

Notifications, clipboard, media controls, wallpaper, theme, and
screenshots.

### Plugins

Plugin event dispatch and plugin/source management.

### System Controls

Volume, microphone, brightness, night light, Wi-Fi, Bluetooth,
caffeine, power profile, and display power.

## Compositor Keybinds

A compositor keybind only needs to launch a Noctalia command. The
command part is compositor-agnostic:

```bash
noctalia msg panel-toggle launcher
noctalia msg panel-toggle control-center
noctalia msg settings-toggle
noctalia msg volume-up
```

Use your compositor's normal `exec`, `spawn`, or `command-binding`
syntax to run those commands.

- [Niri](#niri): Niri bind syntax, overview integration, wallpaper
  backdrop, and blur.
- [Hyprland](#hyprland): Hyprland Lua bind syntax, persistent
  workspaces, and blur.
- [Sway / Scroll](#sway--scroll): Sway-style bind syntax for
  Noctalia launcher, panels, settings, audio, and brightness.
- [Mango](#mango): Mango effects and bind syntax for Noctalia
  launcher, panels, settings, audio, and brightness.

## Discovering Commands

Not sure what else you can control? You can see all IPC commands via
this terminal command:

```bash
noctalia msg --help
```

## Using IPC in Config

Config command fields run through the shell. Use the canonical IPC
CLI when a command needs to call Noctalia, and compose it with other
shell commands when needed:

```toml
[idle.behavior.custom]
timeout = 660
action = "command"
command = "noctalia msg session lock && notify-send 'Session locked'"

[hooks]
started = "noctalia msg panel-toggle launcher"
```
