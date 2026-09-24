# CLI Command Reference

All functions of Appify are fully accessible through the `appify` terminal command.

## Summary of Commands

### Application Management
```bash
# Install a custom web app
appify install https://linear.app --name "Linear" --wm-class "linear-app" --tray

# Launch an app directly by URL or alias
appify run whatsapp
appify run https://music.youtube.com

# List all installed applications
appify list

# Reconfigure an installed app
appify configure whatsapp --zoom 1.1 --hide-on-close

# Clear app cookies and cache without deleting configuration
appify clear-cache whatsapp

# Reinstall app and refresh launcher
appify reinstall whatsapp

# Uninstall app
appify uninstall whatsapp
```

### Presets
```bash
# List available curated presets
appify preset list

# Install a preset
appify preset install discord
appify preset install whatsapp --autostart-hidden
```

### Marketplace
```bash
# List all marketplace extensions
appify marketplace list

# Search marketplace extensions
appify marketplace search "ambient"

# Install an extension from the CDN
appify marketplace install youtube-ambient-control
```

### Graphical Interface & Self-Registration
```bash
# Launch the graphical manager
appify ui

# Register Appify binary into ~/.local/bin and add desktop launcher
appify self-install
```
