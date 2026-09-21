# Appify

Appify is a command-line utility that turns web apps into lightweight, isolated desktop apps powered by
[Tauri](https://tauri.app). Each app runs in its own sandboxed webview — separate cookies, cache, and session
storage — and automatically hands off external links to your system's default browser instead of opening them
in the app window.

---

## Features

* **Isolated storage** — every app gets its own profile directory, keyed by a hash of its URL, so sessions
  never leak between apps or your regular browser.
* **Smart link routing** — navigation within the app's domain stays in the window; anything else (external
  links, `blob:`/`data:` URLs aside) is sent to your default browser instead.
* **Automatic icon fetching** — scrapes the target site for its best available icon (Apple touch icon first,
  falling back to other `<link rel="icon">` tags, then `favicon.ico`) and uses it as the window/launcher icon.
* **Built-in aliases** — short names for popular services (WhatsApp, Discord, Telegram, Spotify, Netflix,
  YouTube, X, Reddit, ChatGPT, Notion, Figma, Gmail) that resolve to the right URL and a sensible default name.
* **Custom window identity** — override the display name, window class (`--wm-class`), and icon (`--icon`) per app.
* **Native desktop integration (Linux)** — `install` writes a `.desktop` launcher and icon so the app shows up
  in your application menu like any other program.

---

## Installation

### Quick Install (Linux & macOS)

Install the latest release with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/larvance/tauri-appify/main/install.sh | bash
```

This automatically detects your OS and CPU architecture, downloads the latest binary directly to `~/.local/bin/appify`, and makes it executable.

---

### Manual Download

Appify is a single self-contained binary — no runtime dependencies to install once you have it in hand.
Grab one from [Releases](../../releases) instead of building it yourself, unless you need a platform/
architecture that isn't published there, in which case see [Build from source](#build-from-source).

> **Automatic Persistence:** Even if you just download the binary to your `~/Downloads` folder and run it (e.g. `./appify_linux_x64 install whatsapp` or `./appify_linux_x64 self-install`), Appify automatically relocates itself to your persistent user binary directory (`~/.local/bin/appify` on Linux/macOS) and registers desktop entries to that permanent location. Clearing your `Downloads` folder will never break your installed desktop web apps.

### Linux

```bash
# Download the release asset for your architecture (e.g. appify_linux_x64), then:
mkdir -p ~/.local/bin
mv appify_linux_x64 ~/.local/bin/appify
chmod +x ~/.local/bin/appify
```

`~/.local/bin` is part of the XDG base directory spec and is already on `PATH` by default on Ubuntu and most
modern distros — no extra setup needed. If `appify` isn't found after this, add it yourself:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### macOS

```bash
# Download the release asset for your chip:
#   Apple Silicon: appify_darwin_aarch64
#   Intel:         appify_darwin_x64
sudo mv appify_darwin_<arch> /usr/local/bin/appify
sudo chmod +x /usr/local/bin/appify
```

`/usr/local/bin` is a standard Unix location that's on `PATH` by default on macOS, with or without Homebrew.

Because the binary isn't notarized by Apple, Gatekeeper will refuse to run it on first launch ("cannot be
opened because the developer cannot be verified"). Clear the quarantine flag once, after copying it in:

```bash
xattr -d com.apple.quarantine /usr/local/bin/appify
```

### Windows

1. Download the `appify_windows_x64.exe` release asset for your system.
2. Create a folder to hold it, e.g. `%USERPROFILE%\bin`, and move the file there as `appify.exe`.
3. Add that folder to your user `PATH` (one-time):
   ```powershell
   setx PATH "%PATH%;%USERPROFILE%\bin"
   ```
   Or via the GUI: **Settings → System → About → Advanced system settings → Environment Variables**, edit
   the `Path` variable under "User variables", and add the folder.
4. Open a **new** terminal window for the `PATH` change to take effect.

Since the binary is unsigned, Windows SmartScreen may show a warning the first time you run it. Click
**More info → Run anyway** to proceed.

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

Then place the resulting binary in the same location described in [Installation](#installation) for your OS
(e.g. on Linux: `mkdir -p ~/.local/bin && cp target/release/appify ~/.local/bin/`).

> **Note:** `install` currently generates a Linux-style `.desktop` entry only, regardless of what platform you
> build on — see [Platform support](#platform-support).

---

## Usage

```text
appify install <URL or alias> [NAME] [OPTIONS]
appify run     <URL or alias> [NAME] [OPTIONS]
appify uninstall <URL or alias>
appify list
appify self-install
```

* `install` — writes a desktop launcher so the app is available from your app menu, and downloads/caches its icon. Persists any runtime options into the desktop launcher. Automatically ensures `appify` is installed to `~/.local/bin/appify`.
* `run` — opens the app directly in an isolated window without touching your desktop launchers (useful for
  testing, or as the command the generated `.desktop` file itself calls).
* `uninstall` — removes the launcher, icon, and cached session profile for a given app.
* `list` — prints every app currently installed via Appify.
* `self-install` — installs the `appify` binary itself to `~/.local/bin/appify` (or system PATH).

### Options & Flags

Both `install` and `run` accept the following customization options (when passed to `install`, they are automatically serialized into the `.desktop` launcher):

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
| `--width <WIDTH>` | Initial window width in pixels. |
| `--height <HEIGHT>` | Initial window height in pixels. |
| `-a, --allow-domain <DOMAIN>` | Additional internal domains to keep in-app (e.g. `-a whatsapp.net`). Can be repeated. |
| `--wm-class <CLASS>` | Custom window class / App ID for desktop grouping and dock matching. |
| `--icon <PATH_OR_URL>` | Custom window and desktop launcher icon. |

### Installing an app

```bash
# Uses the built-in alias, default name "WhatsApp"
appify install whatsapp

# Complete messenger setup: autostarts silently into tray on boot, single instance, close hides to tray
appify install whatsapp --single-instance --hide-on-close --autostart-hidden

# Override display name and window size
appify install discord "Discord" --single-instance --width 1280 --height 800

# Full control: custom name, window class, and icon
appify install https://mail.google.com "Gmail" --wm-class gmail-app --icon ./gmail.png
```

#### Built-in aliases

| Alias           | Target URL                 | Default name |
|-----------------|----------------------------|--------------|
| `whatsapp`      | `https://web.whatsapp.com` | WhatsApp     |
| `discord`       | `https://discord.com/app`  | Discord      |
| `telegram`      | `https://web.telegram.org` | Telegram     |
| `spotify`       | `https://open.spotify.com` | Spotify      |
| `netflix`       | `https://www.netflix.com`  | Netflix      |
| `youtube`       | `https://youtube.com`      | YouTube      |
| `twitter` / `x` | `https://x.com`            | X            |
| `reddit`        | `https://reddit.com`       | Reddit       |
| `chatgpt`       | `https://chatgpt.com`      | ChatGPT      |
| `notion`        | `https://notion.so`        | Notion       |
| `figma`         | `https://figma.com`        | Figma        |
| `gmail`         | `https://mail.google.com`  | Gmail        |

#### Using any URL

Anything that isn't a recognized alias is treated as a raw URL. `https://` is added automatically if you leave
off a scheme, and the display name defaults to the hostname if you don't provide one:

```bash
# Defaults to display name "github.com"
appify install github.com

# Explicit display name
appify install https://github.com "GitHub"
```

If a hostname doesn't look like a valid domain (no dot, and not `localhost`), Appify will reject it with an error.

### Running an app without installing it

```bash
appify run discord
appify run https://linear.app "Linear" --wm-class linear-app
```

### Listing installed apps

```bash
appify list
```

```text
App Name                  | WM_CLASS             | URL
---------------------------------------------------------------------------
WhatsApp                  | whatsapp-desktop      | https://web.whatsapp.com
GitHub                    | appify-3f9a2c1b0e4d   | https://github.com
```

### Uninstalling an app

Pass the same alias or URL you used to install it — Appify re-derives the same identity hash and removes the
matching launcher, icon, and session profile:

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
