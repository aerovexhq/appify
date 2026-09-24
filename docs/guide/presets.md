# Curated Presets

Appify includes built-in curated presets for popular productivity, communication, and media applications.

## Included Presets

| Preset | Target URL | Built-in Enhancements |
| :--- | :--- | :--- |
| **WhatsApp Web** | `https://web.whatsapp.com` | Automatically hides promo banners ("Get WhatsApp for Mac/Windows"), enables clipboard image drag/paste, and retains notifications. |
| **Discord** | `https://discord.com/app` | Streamlined tray integration, hide-on-close, custom zoom, and smooth hardware scaling. |
| **Telegram** | `https://web.telegram.org` | Fast and secure messaging client with tray icon, background notifications, and media streaming. |
| **Spotify** | `https://open.spotify.com` | Dedicated web audio client with media playback, tray controls, and hardware key integration. |

## Installing Presets

### Via Graphical Manager
1. Launch `appify ui`.
2. Click on the **Preset Store** tab in the sidebar.
3. Click **1-Click Install** on WhatsApp, Discord, Telegram, or Spotify.

### Via Command Line
```bash
# Install WhatsApp preset
appify preset install whatsapp

# Install Telegram preset
appify preset install telegram

# Install Spotify preset with tray and autostart
appify preset install spotify --tray --autostart

# Install Discord preset with system tray enabled
appify preset install discord --tray --hide-on-close
```
