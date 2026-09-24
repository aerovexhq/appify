# Appify

Appify is a command-line utility that turns web apps into lightweight, isolated desktop apps powered by
[Tauri](https://tauri.app). Each app runs in its own sandboxed webview — separate cookies, cache, and session
storage — and automatically hands off external links to your system's default browser instead of opening them
in the app window.

---

## Features

* **Modern Embedded GUI Manager** — launch `appify` or `appify ui` to open a sleek, OS-independent glassmorphic management dashboard to browse presets, manage installed apps, reconfigure settings, and edit scripts.
* **Curated Preset Store** — one-click and single-command installation for optimized presets (**WhatsApp** and **Discord**) with pre-tuned domains, window dimensions, single-instance locks, and tray behavior.
* **User Scripts & Styles Injection** — customize web apps using `--inject-js` and `--inject-css` (or drop `userscript.js` / `userstyle.css` directly into the app profile, or edit them live in the GUI Manager). Zero-FOUC native injection on Linux via WebKit UserStyleSheet.
* **Live Reconfiguration** — tweak window dimensions, zoom, tray settings, autostart, and injected scripts (`appify configure <app> ...` or via GUI) without wiping session logins or cookies.
* **Cache Management** — reset corrupted browser cache and cookies with `appify clear-cache <app>` while safely preserving app configuration, desktop shortcuts, and custom scripts.
* **Isolated storage** — every app gets its own profile directory, keyed by a SHA-256 hash of its URL, so sessions never leak between apps or your regular browser.
* **Clipboard Image Pasting** — native Linux/WebKitGTK clipboard bridge ensuring images copied from screenshots, image editors, or file managers paste seamlessly into web apps (WhatsApp, Discord, etc.).
* **DevTools & Inspection** — press <kbd>F12</kbd> or <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>I</kbd>/<kbd>C</kbd>/<kbd>J</kbd>, or right-click to inspect, or use the system tray menu to toggle developer tools.
* **Smart link routing** — navigation within the app's domain stays in the window; external links open in your default browser.
* **Automatic icon fetching** — scrapes high-resolution square icons and converts them to crisp PNG desktop launcher icons.
* **Native desktop integration** — generates `.desktop` launchers, icon caches, and system autostart entries (`~/.config/autostart`).

---

## Installation

### Setup Wizards & Installers (Recommended)

Appify provides native installation wizards and packages for all major operating systems. Grab the appropriate installer from the latest [GitHub Releases](../../releases):

#### Windows
* **[Setup Wizard (`.exe`)](../../releases/latest)** — Interactive NSIS setup wizard that installs Appify, creates Start Menu and Desktop shortcuts, adds Appify to your user `PATH`, and includes a clean uninstaller.
* **[Windows Installer (`.msi`)](../../releases/latest)** — Standard WiX MSI package designed for system administrators and enterprise deployment.

#### macOS
* **[Apple Disk Image (`.dmg`)](../../releases/latest)** — Visual setup wizard with drag-to-Applications layout:
  * **Apple Silicon (M1/M2/M3/M4):** `appify_*_aarch64.dmg`
  * **Intel:** `appify_*_x64.dmg`
* **Application Bundle (`.app.tar.gz`)** — Pre-packaged `.app` bundle.

#### Linux
* **[Debian / Ubuntu (`.deb`)](../../releases/latest)** — Native package for Ubuntu, Debian, Linux Mint, Pop!_OS, and elementary OS:
  ```bash
  sudo apt install ./appify_*_amd64.deb
  ```
* **[Universal AppImage (`.AppImage`)](../../releases/latest)** — Portable, self-contained executable that runs on any modern Linux distribution without installation:
  ```bash
  chmod +x appify_*_amd64.AppImage
  ./appify_*_amd64.AppImage
  ```
* **[Fedora / RHEL / openSUSE (`.rpm`)](../../releases/latest)** — Native RPM package:
  ```bash
  sudo dnf install ./appify-*.x86_64.rpm
  ```

---

### Quick CLI Install (Linux & macOS)

If you prefer installing directly from your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/larvance/tauri-appify/main/install.sh | bash
```

This automatically detects your OS and CPU architecture, downloads the latest binary directly to `~/.local/bin/appify`, and makes it executable.

---

### Portable Standalone Binaries

If you prefer a single portable executable without running an installer, download the plain binary for your platform from [Releases](../../releases):
* Linux: `appify_linux_x64`
* macOS (Apple Silicon): `appify_darwin_aarch64`
* macOS (Intel): `appify_darwin_x64`
* Windows: `appify_windows_x64.exe`

> **Automatic Persistence:** Even if you download the binary to your `~/Downloads` folder and run it (e.g. `./appify_linux_x64 install whatsapp` or `./appify_linux_x64 self-install`), Appify automatically relocates itself to your persistent user binary directory (`~/.local/bin/appify` on Linux/macOS) and registers desktop entries to that permanent location. Clearing your `Downloads` folder will never break your installed desktop web apps.

---

## Build from source

Build from source if you're on a platform/architecture without a published release, or want to build from a
specific commit.

### 1. Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Platform system libraries

#### Linux (Debian / Ubuntu 24.04+)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### macOS

```bash
xcode-select --install
```

#### Windows

Install the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) via the
Visual Studio Installer, selecting the "Desktop development with C++" workload.

### 3. Build and install

```bash
git clone https://github.com/<your-username>/tauri-appify.git
cd tauri-appify
cargo build --release
```

Then place the resulting binary in `~/.local/bin/appify` (or run `cargo run --release -- self-install`).

---

## Usage

### Graphical Manager (GUI)

Simply run `appify` with no arguments, or click the **Appify** launcher in your application menu:

```bash
appify
# or
appify ui
```

This launches the Appify Management UI where you can:
* Browse and install curated presets (**WhatsApp**, **Discord**).
* View all installed apps with live status indicators.
* Reconfigure window sizes, zoom, autostart, and tray behavior on the fly.
* Write and test custom user scripts (JavaScript) and custom styles (CSS) with live code editors.
* Clear session caches without losing your configurations.
* Install custom web apps by entering a URL and friendly name.

---

### Command-Line Interface (CLI)

```text
appify [COMMAND]

Commands:
  ui, gui             Launch the Appify Manager GUI (default if no command given)
  preset list         Browse built-in curated app presets
  preset install      Install a preset by name (e.g. whatsapp, discord)
  install             Install a custom website or alias as a desktop app
  run                 Run a web app directly in an isolated window without installing
  configure           Reconfigure settings or scripts for an installed app without losing data
  reinstall           Re-generate launcher and icon from fresh web scrape
  clear-cache         Clear session cookies and webview cache while preserving config
  uninstall           Remove app desktop entry, icons, autostart, and profile data
  list                List all currently installed apps
  self-install        Install appify binary to ~/.local/bin and register manager desktop entry
```

### Options & Flags

The `install`, `run`, and `configure` commands accept the following flags:

| Option | Description |
|---|---|
| `--autostart` | Starts the application automatically upon user login / system boot. |
| `--autostart-hidden` | Starts the app on login directly in the background / system tray. Regular launcher clicks still open the window. |
| `--hide-on-close` | Closes hide the window to background instead of quitting (tray icon automatically enabled). |
| `--single-instance` | Restricts to one running instance. Launching a duplicate unhides and focuses the existing window. |
| `--tray` | Shows a system tray icon with Show/Hide, Reload, and Quit actions, plus click-to-toggle. |
| `--start-hidden` | Starts minimized/hidden to background tray. |
| `--maximize` | Launches the app in a maximized window. |
| `--zoom <ZOOM>` | Initial webview zoom scale factor (e.g. `1.1`, `0.9`). |
| `--user-agent <UA>` | Custom browser User-Agent string. |
| `--width <WIDTH>` | Initial window width in pixels (e.g. `1280`). |
| `--height <HEIGHT>` | Initial window height in pixels (e.g. `850`). |
| `-a, --allow-domain <DOMAIN>` | Additional internal domains to keep in-app (e.g. `-a cdn.discordapp.com`). Can be repeated. |
| `--wm-class <CLASS>` | Custom window class / App ID for desktop grouping and dock matching. |
| `--icon <PATH_OR_URL>` | Custom window and desktop launcher icon. |
| `--inject-js <JS>` | Custom JavaScript string or path to `.js` file to inject on app load. |
| `--inject-css <CSS>` | Custom CSS string or path to `.css` file to inject into the webview. |

---

### Presets

List available presets:
```bash
appify preset list
```

Install a preset with its recommended defaults:
```bash
appify preset install whatsapp
appify preset install discord
```

You can customize presets on install by passing additional flags:
```bash
appify preset install whatsapp --autostart-hidden --zoom 1.1
```

---

### Installing Custom Apps

```bash
# Uses built-in alias with sensible defaults
appify install telegram

# Messenger app with full background tray and single-instance behavior:
appify install whatsapp --single-instance --hide-on-close --autostart-hidden

# Any custom URL with custom window size
appify install https://github.com "GitHub" --width 1440 --height 900

# App with custom script and CSS injection
appify install https://news.ycombinator.com "HackerNews" \
  --inject-css "body { font-family: sans-serif !important; max-width: 900px; margin: auto; }" \
  --inject-js "console.log('Appify custom script loaded');"
```

---

### Reconfiguring an Installed App

Change zoom, window size, autostart, or scripts without losing your login session or cookies:

```bash
# Update Discord zoom and add a custom CSS theme
appify configure discord --zoom 1.1 --inject-css ./my-discord-theme.css

# Enable autostart in background for WhatsApp
appify configure whatsapp --autostart-hidden
```

---

### Clearing Cache

If an app's webview cache gets corrupted or you want to force re-login, clear its cache without deleting its configuration or custom scripts:

```bash
appify clear-cache discord
```

---

### Uninstalling an App

Pass the same alias, URL, or display name to completely remove the launcher, icon, autostart entry, and isolated profile:

```bash
appify uninstall whatsapp
appify uninstall https://github.com
```

---

## How identity works

Each app is identified by a SHA-256 hash of its fully-resolved URL. That hash is used to:

* namespace the app's isolated session/profile directory,
* name its cached icon and `.desktop` file, and
* generate a default `--wm-class` (`appify-<first 12 hex chars>`) when one isn't supplied.

Because the hash is derived from the URL, `install`, `run`, and `uninstall` all agree on the same identity as
long as you pass the same URL or alias — no separate ID file to track.

---

## File system locations

| Component             | Location (Linux)                                                                          |
|-----------------------|-------------------------------------------------------------------------------------------|
| Desktop launcher      | `~/.local/share/applications/appify-<hash>.desktop`                                       |
| Icon                  | `~/.local/share/icons/appify-<hash>.png`                                                  |
| Isolated session data | `$XDG_DATA_HOME/appify/profiles/<hash>` (usually `~/.local/share/appify/profiles/<hash>`) |
| Binary                | wherever you copy it — `~/.local/bin/` is the convention used above                       |

---

## Platform support

The isolated webview, link routing, and icon fetching are all built on cross-platform Tauri APIs and compile on
Linux, macOS, and Windows. As of this release, though, `install` only generates a Linux `.desktop` entry;
`appify run` works everywhere. macOS `.app` bundles and Windows Start Menu shortcuts aren't wired up yet — if
you'd like to use Appify as a launcher on those platforms today, `run` it directly or wrap it in your own
shortcut/alias.
