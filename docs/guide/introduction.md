# Introduction to Appify

**Appify** is a lightweight, modern runtime and desktop manager that turns any website or web application into a native desktop application.

Unlike traditional wrappers built with Electron, Appify uses **Tauri v2** and native system web engines (**WebKitGTK** on Linux) written in Rust. This results in:
- **Fractional RAM usage**: ~30MB vs 400MB+ for Electron.
- **Tiny binary footprint**: Self-contained runtime under 20MB.
- **True process isolation**: Completely separate data partitions, cookies, local storage, and caches.

## Why Appify?

### Complete Multi-Profile Separation
Each application is assigned a unique deterministic SHA-256 hash derived from its identity:
```text
~/.local/share/appify/<hash>/
├── app.json         # Complete configuration (window sizing, zoom, toggles)
├── userscript.js    # Compiled active JavaScript plugins
├── userstyle.css    # Compiled active CSS themes
└── webkit/          # Isolated session cookies, cache, and IndexedDB
```
You can run multiple instances of the same service (e.g., personal vs. work accounts) without cookie collision.

### Modern Graphical Dashboard & Powerful CLI
Appify provides both a glassmorphism desktop GUI (`appify ui`) and a comprehensive CLI for power users and automation scripts.

### Extension Studio
Inject custom user scripts and styles with full Monaco Editor integration, hold-to-repeat number steppers, and drag-and-drop sequence reordering.
