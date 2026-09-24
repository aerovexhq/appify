# Installation Guide

Appify supports multiple installation formats across Linux distributions.

## One-Line Installer (Recommended)

Run the automated installer script:
```bash
curl -fsSL https://raw.githubusercontent.com/aerovexsim/appify/main/install.sh | bash
```

## Debian / Ubuntu (.deb)

Download the `.deb` package from the [Releases](https://github.com/aerovexsim/appify/releases) page:
```bash
sudo dpkg -i appify_*_amd64.deb
sudo apt-get install -f
```

## Universal AppImage

Download `appify_*_amd64.AppImage` from GitHub Releases:
```bash
chmod +x appify_*_amd64.AppImage
./appify_*_amd64.AppImage
```

## Building from Source (Cargo)

Prerequisites on Debian/Ubuntu:
```bash
sudo apt update
sudo apt install -y build-essential curl libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

Build and install:
```bash
git clone https://github.com/aerovexsim/appify.git
cd appify
cargo build --release
install -m 755 target/release/appify ~/.local/bin/appify
```

Verify your installation:
```bash
appify --version
```
