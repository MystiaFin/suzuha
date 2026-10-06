<div align="center">
  <h1>suzuha</h1>
  <p><strong>My amane config.</strong><br>
  An opinionated Wayland desktop shell written in Rust with amane, built for Niri.</p>

  <p>
    <img alt="amane 0.1" src="https://img.shields.io/badge/AMANE-0.1-89b4fa?style=flat-square&labelColor=181825">
    <img alt="Niri" src="https://img.shields.io/badge/COMPOSITOR-NIRI-89b4fa?style=flat-square&labelColor=181825">
    <img alt="Rust" src="https://img.shields.io/badge/LANGUAGE-RUST-89b4fa?style=flat-square&labelColor=181825">
  </p>
</div>

## Preview

<!-- drop the preview video link here -->

https://github.com/user-attachments/assets/VIDEO-ID

## Overview

suzuha is my personal amane configuration: a full desktop shell in Rust rather than a pile of separate widgets. It started as a rewrite of my [JAQC-shell](https://github.com/MystiaFin/shell) Quickshell config, so the two look and feel pretty much the same.

### Included

- Status bar with workspaces, system information, clock, media, and a pomodoro pill
- Liquid shader that lets the bar and panels flow into each other, with rounded screen corners
- Application launcher with a `>` command mode and a tmux project picker
- Control center with media controls and a cava visualizer
- Utility panel with Wi-Fi, Bluetooth, brightness, calendar, notifications, and a screen recorder
- Wallpaper picker with animated transitions and shuffle
- Wallpaper-derived dynamic color palette, plus fixed Gruvbox and Catppuccin schemes
- Floating desktop widgets that place themselves around the wallpaper
- Pomodoro timer
- File converter for images, video, audio, and documents
- Power menu
- logind-driven Wayland lock screen
- Built-in settings window
- Optional theme integrations for GTK, terminals, tmux, Vesktop, Spotify, btop, and cava

## Installation

Clone the repository as your amane config:

```sh
git clone https://github.com/MystiaFin/suzuha.git ~/.config/amane
```

Build the shell, then launch it:

```sh
amane compile
amane run
```

`amane run` only starts the compiled shell, so run `amane compile` again after pulling changes. While editing the config, `amane dev` rebuilds and restarts it on every save.

Or start it with Niri:

```kdl
spawn-at-startup "amane" "run"
```

> [!NOTE]
> This config is built around Niri. Some parts, like the power menu and the wallpaper sitting behind the overview, expect Niri and are not expected to work unchanged on other compositors.

## Dependencies

### Required

- **amane**
- **Niri**
- **Poppins**
- **JetBrains Mono Nerd Font**
- **Symbols Nerd Font**
- **Material Design Icons**
- **curl**

### Optional

These are only needed for their corresponding features:

| Package | Used for |
| --- | --- |
| `cava` | Media visualizer |
| `ffmpeg` / `ffprobe` | Image, video, and audio conversion |
| `ImageMagick 7` | Image conversion |
| `LibreOffice` (`soffice`) | Document conversion |
| `pw-play` / PipeWire | Pomodoro sounds |
| `libnotify` (`notify-send`) | Pomodoro notifications |
| `kitty` remote control | Live kitty palette updates |
| `dconf` | GTK theme switching |
| `tmux` | Project launcher and generated tmux palette |
| `xdg-desktop-portal` | Picking a profile picture |
| `wf-recorder` | Screen recording |
| `pactl` | Recording desktop sound and mic together |

## IPC

Everything is driven through `amane ipc call`, so it is easy to bind in Niri:

```kdl
Mod+Space { spawn "amane" "ipc" "call" "launcher" "toggle"; }
Mod+Shift+W { spawn "amane" "ipc" "call" "wallpaper" "toggle"; }
```

<details>
<summary><strong>All IPC targets</strong></summary>

```sh
amane ipc call launcher toggle     # also show, hide, showTmux
amane ipc call utility toggle      # also show, hide, then a page: notifications, wifi, bluetooth, record
amane ipc call control toggle      # also show, hide
amane ipc call wallpaper toggle    # also show, hide
amane ipc call settings
amane ipc call record              # start or stop a screen recording
amane ipc call converter
amane ipc call lock
```

</details>

## Launcher

The launcher handles normal application search and also acts as a small command palette.

Type `>` to switch into command mode. Commands can expose things such as:

- Settings
- Wallpapers
- Tmux sessions
- Other shell actions

The available command entries can be enabled or disabled from **Settings → Launcher**.

`showTmux` opens the launcher straight into the tmux project picker:

```kdl
Mod+Shift+P { spawn "amane" "ipc" "call" "launcher" "showTmux"; }
```

## Wallpapers & colors

By default the wallpaper picker reads images from:

```text
~/Pictures/Wallpapers
```

The folder can be changed from **Settings → Wallpaper**.

The selected wallpaper is persisted in:

```text
~/.local/state/amane/wallpaper-selection
```

The wallpaper can also drive the shell's dynamic palette. Wallpaper transitions, shuffle, light/dark mode, and color schemes are configurable from the settings window.

## Lock screen

suzuha includes a Wayland session lock that listens to logind, so anything that asks logind to lock (`loginctl lock-session`, an idle daemon, closing the lid) brings it up. It reuses the active wallpaper and palette, and slides the password field up once you start typing.

Lock it from the power menu, or through IPC:

```sh
amane ipc call lock
```

The name and profile picture shown on the lock screen can be changed under **Settings → User**; they do not change the system account used for authentication.

> [!CAUTION]
> A Wayland session lock deliberately stays locked if the locker dies. If amane crashes while the session is locked, the compositor will not reveal the desktop; recover from another TTY if necessary.

## Pomodoro & converter

Two small tools that live inside the shell instead of being separate apps:

- **Pomodoro** — a focus timer with a wavy progress ring, also shown as a pill on the bar
- **Converter** — converts and compresses images, video, audio, and documents through `ffmpeg`, ImageMagick, and LibreOffice

```sh
amane ipc call converter
```

## Settings

The shell includes its own settings window, so normal day-to-day preferences don't need code changes.

Current sections include:

- Appearance
- User
- Colors
- Wallpaper
- Bar
- Launcher
- Behavior
- Floating widgets
- Weather
- Integrations
- About

Settings are stored in:

```text
~/.local/state/amane/settings
```

## Theme integrations

External theme integrations are **opt-in**. Enabling one may generate configuration files or update a running application, so the shell does not enable them automatically.

Supported integrations currently include:

- GTK 3 / GTK 4
- kitty
- foot
- tmux
- Vesktop
- Spotify / Spicetify
- btop
- cava

<details>
<summary><strong>Generated files and side effects</strong></summary>

Depending on which integrations are enabled, suzuha may write files like:

```text
~/.local/state/amane/terminal-colors-kitty.conf
~/.local/state/amane/terminal-colors-foot.ini
~/.local/state/amane/tmux-colors.conf
~/.config/btop/themes/amane.theme
~/.config/cava/themes/amane
~/.cache/amane/spotify.css
```

GTK integration also generates light and dark wallpaper-derived themes under `~/.local/share/themes/` and updates the active color-scheme preference through `dconf`.

For tmux, add this to `~/.tmux.conf` so new sessions load the generated palette:

```tmux
source-file -q ~/.local/state/amane/tmux-colors.conf
```

For kitty, include the generated colors and let running windows be recolored live:

```conf
allow_remote_control socket-only
listen_on unix:@amane-kitty
include ~/.local/state/amane/terminal-colors-kitty.conf
```

</details>

## Weather

The floating weather widget's location is set under **Settings → Weather** with a place name, latitude, and longitude.

Using approximate city-center coordinates is enough. Weather and air quality data are fetched directly from Open-Meteo; no IP geolocation service is used.

## Project structure

```text
src/bar/            status bar, workspaces, pills, and the pomodoro ring
src/overlay/        launcher, control center, utility panel, power menu
src/floating/       floating desktop widgets and their placement
src/wallpaper/      wallpaper, picker, transitions, and shuffle
src/lock_screen/    lock screen and the logind listener
src/settings/       settings window and its pages
src/theme/          palette generation and color schemes
src/integrations/   external theme/application integrations
src/converter/      file converter
src/motion/         springs, glides, and shared animation helpers
shaders/            the liquid shader
src/main.rs         root configuration
```

## Notes

This is a personal config that I daily-drive and keep changing. Expect opinions, occasional breakage, and features that exist because I wanted them on my own desktop.

If you use it as a base for your own setup, reading and modifying the Rust is very much part of the experience.
