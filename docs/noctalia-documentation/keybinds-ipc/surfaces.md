# Surfaces

These commands open, close, or adjust Noctalia UI surfaces.

## Bar

| Action | Command | Description |
| --- | --- | --- |
| Show bar | `bar-show [bar-name] [monitor-selector]` | Reveal matching bar instance(s). Omit both arguments to show every bar on every output. |
| Hide bar | `bar-hide [bar-name] [monitor-selector]` | Hide matching bar instance(s) and block edge/pointer reveal until the next show/toggle. Omit both arguments to hide every bar. |
| Toggle bar | `bar-toggle [bar-name] [monitor-selector]` | Toggle visibility for matching bar instance(s). Omit both arguments to toggle every bar. Does not block edge reveal on hide. |
| Toggle reserve space | `bar-reserve-toggle [bar-name] [monitor-selector]` | Toggle reserve space for matching bar instance(s). Omit both arguments to toggle every bar. |
| Set bar auto-hide | `bar-auto-hide-set <on\|off\|smart> [bar-name] [monitor-selector]` | Temporarily enable, disable or switch to smart auto-hide for matching bar instance(s). Omit bar-name to update every bar. |
| Set bar layer | `bar-layer-set <top\|overlay> [bar-name] [monitor-selector]` | Temporarily move matching bar instance(s) to the top or overlay layer-shell layer. `overlay` shows the bar above fullscreen apps; attached panels follow the bar's layer. Omit bar-name to update every bar. |

> **Note:** Use `bar-hide` when you want the bar to stay hidden
> until `bar-show`.

## Panels

| Action | Command | Description |
| --- | --- | --- |
| Open panel | `panel-open <id> [context]` | Open a panel by id without toggling it closed when it is already open. Optional context works like `panel-toggle <id> [context]`. |
| Close panel | `panel-close [id]` | Close the active panel, or close the named panel if it is active. |
| Toggle launcher | `panel-toggle launcher [query]` | Open or close the app launcher. Optional query pre-fills the search input (e.g. `noctalia msg panel-toggle launcher "/wall"`). |
| Toggle session menu | `panel-toggle session` | Open or close the logout, reboot, and shutdown menu. |
| Toggle clipboard | `panel-toggle clipboard` | Open or close clipboard history. |
| Toggle wallpaper panel | `panel-toggle wallpaper` | Open or close wallpaper picker. |
| Toggle control center | `panel-toggle control-center [tab]` | Open or close Control Center. Optional tab opens the tab inside control center by default (e.g. `noctalia msg panel-toggle control-center media`). |
| Toggle plugin panel | `panel-toggle <author/plugin:entry> [context]` | Open or close a plugin `[[panel]]` entry (e.g. `noctalia msg panel-toggle noctalia/wallhaven:browser`). Optional context is passed to `onOpen`. |

## Dock

| Action | Command | Description |
| --- | --- | --- |
| Show dock | `dock-show` | Show all dock instances and save that override. |
| Hide dock | `dock-hide` | Hide all dock instances and save that override. |
| Toggle dock | `dock-toggle` | Toggle dock visibility and save the new state. |
| Reload dock | `dock-reload` | Reload dock configuration. |

## Desktop Widgets

| Action | Command | Description |
| --- | --- | --- |
| Enter edit mode | `desktop-widgets-edit` | Start moving, resizing, rotating, and removing desktop widgets. |
| Exit edit mode | `desktop-widgets-exit` | Leave desktop widget edit mode. |
| Toggle edit mode | `desktop-widgets-toggle-edit` | Switch edit mode on or off. |
| Show widgets | `desktop-widgets-show` | Reveal desktop widgets now, even when the saved setting is disabled. Runtime only - does not change the saved setting, and resets on restart. |
| Hide widgets | `desktop-widgets-hide` | Hide desktop widgets now, tearing down their instances so rendering and compute stop. Runtime only - does not change the saved setting, and resets on restart. |
| Toggle widgets | `desktop-widgets-toggle` | Flip current visibility; prints `shown` or `hidden`. Runtime only - does not change the saved setting. Changing the saved Desktop > Widgets toggle cancels the override. |

## Lockscreen Widgets

| Action | Command | Description |
| --- | --- | --- |
| Enter edit mode | `lockscreen-widgets-edit` | Edit lockscreen widget layout (session must be unlocked). |
| Exit edit mode | `lockscreen-widgets-exit` | Leave lockscreen widget edit mode. |
| Toggle edit mode | `lockscreen-widgets-toggle-edit` | Switch lockscreen widget edit mode on or off. |
