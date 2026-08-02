# Shell

These commands control shell state, logging, settings windows, the
window switcher, and session actions.

## General

| Action | Command | Description |
| --- | --- | --- |
| Print status | `status` | Print basic shell state as JSON. |
| Show log level | `log-level-status` | Print the current console log level. |
| Set log level | `log-level-set debug` | Set the console log level for the running session. Use `debug`, `info`, `warn`, or `error`. |
| Reload config | `config-reload` | Reload the merged Noctalia config stack immediately. |
| Open settings | `settings-open [context]` | Open or focus the graphical settings window, optionally at a specific section (e.g. `bar`). |
| Open plugin settings | `settings-open-plugin noctalia/notes` | Open the settings window at an installed plugin's settings. The plugin must be enabled and expose settings. |
| Close settings | `settings-close` | Close the graphical settings window. |
| Toggle settings | `settings-toggle [context]` | Open or close the graphical settings window, optionally at a specific section. |

> **Note:** Set `NOCTALIA_LOG_LEVEL` to `debug`, `info`, `warn`, or
> `error` before startup to choose the initial console log level.
> The default is `info`; the file log remains unfiltered.

## Window Switcher

Alt+Tab-style overlay: a dimmed fullscreen layer with a centered
grid of open windows (application icon, title, optional app name).
There is no `[shell.window_switcher]` configuration block yet -
wire your compositor shortcut to the IPC commands below.

| Action | Command | Description |
| --- | --- | --- |
| Open | `window-switcher` | Show the switcher on the preferred monitor (pointer, keyboard focus, or first output). |
| Close | `window-switcher close` | Hide the overlay without changing focus. `hide` is an alias. |

Example Hyprland bind:

```lua
hl.bind("ALT + TAB", hl.dsp.exec_cmd("noctalia msg window-switcher"))
```

While the overlay is open it takes exclusive keyboard focus on its
layer surface:

- **Tab / Shift+Tab** - cycle selection
- **Arrow keys** - move within the grid (up to five columns)
- **Enter** - focus the selected window and close
- **Escape** - close without switching
- **Pointer** - click a tile to switch; click the close control on
  a tile to close that window

The window list refreshes while the overlay stays open (new
windows, closed windows, and title changes such as Discord channel
switches). Selection stays on the same window when its title
updates.

## Session

| Action | Command | Description |
| --- | --- | --- |
| Lock session | `session lock` | Lock the current session. `loginctl lock-session` uses the same path when Noctalia is running. |
| Suspend system | `session suspend` | Suspend without locking first. |
| Lock and suspend | `session lock-and-suspend` | Lock the session, then suspend once the lock is active. |
| Log out | `session logout` | End the graphical session (compositor-native when available). |
| Reboot | `session reboot` | Reboot the system. |
| Shut down | `session shutdown` | Shut down the system. |
