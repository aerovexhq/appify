use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;
use url::Url;

#[derive(Parser)]
#[command(name = "appify", version, about = "Convert web apps into isolated desktop apps", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a web app (creates desktop entry and fetches icon)
    Install {
        /// Target URL or alias (e.g. whatsapp)
        url: String,

        /// Display Name for window and launcher
        name: Option<String>,

        /// Window class / App ID for desktop grouping
        #[arg(long)]
        wm_class: Option<String>,

        /// Custom icon path or URL (defaults to auto-fetching from website)
        #[arg(long)]
        icon: Option<String>,

        /// Additional domains to keep inside the app (can be repeated, e.g. -a whatsapp.net)
        #[arg(short = 'a', long = "allow-domain")]
        allow_domain: Vec<String>,

        /// Keep app running in background when window is closed
        #[arg(long)]
        hide_on_close: bool,

        /// Enforce only one running instance of this app
        #[arg(long)]
        single_instance: bool,

        /// Show system tray icon for managing show/hide and reload
        #[arg(long)]
        tray: bool,

        /// Launch the window hidden in background / tray
        #[arg(long)]
        start_hidden: bool,

        /// Launch the window maximized
        #[arg(long)]
        maximize: bool,

        /// Initial zoom level (e.g. 1.0, 1.1, 0.9)
        #[arg(long)]
        zoom: Option<f64>,

        /// Override browser User-Agent string
        #[arg(long)]
        user_agent: Option<String>,

        /// Initial window width
        #[arg(long)]
        width: Option<f64>,

        /// Initial window height
        #[arg(long)]
        height: Option<f64>,

        /// Launch app automatically on user login
        #[arg(long)]
        autostart: bool,

        /// Launch app automatically on user login in background/tray
        #[arg(long)]
        autostart_hidden: bool,

        /// Inject custom JavaScript (file path or code)
        #[arg(long)]
        inject_js: Option<String>,

        /// Inject custom CSS stylesheet (file path or code)
        #[arg(long)]
        inject_css: Option<String>,
    },
    /// Run an app directly in an isolated webview
    Run {
        /// Target URL or alias
        url: String,

        /// Display Name for the window
        name: Option<String>,

        /// Window class / App ID for desktop grouping
        #[arg(long)]
        wm_class: Option<String>,

        /// Path to window icon
        #[arg(long)]
        icon: Option<String>,

        /// Additional domains to keep inside the app (can be repeated, e.g. -a whatsapp.net)
        #[arg(short = 'a', long = "allow-domain")]
        allow_domain: Vec<String>,

        /// Keep app running in background when window is closed
        #[arg(long)]
        hide_on_close: bool,

        /// Enforce only one running instance of this app
        #[arg(long)]
        single_instance: bool,

        /// Show system tray icon for managing show/hide and reload
        #[arg(long)]
        tray: bool,

        /// Launch the window hidden in background / tray
        #[arg(long)]
        start_hidden: bool,

        /// Launch the window maximized
        #[arg(long)]
        maximize: bool,

        /// Initial zoom level (e.g. 1.0, 1.1, 0.9)
        #[arg(long)]
        zoom: Option<f64>,

        /// Override browser User-Agent string
        #[arg(long)]
        user_agent: Option<String>,

        /// Initial window width
        #[arg(long)]
        width: Option<f64>,

        /// Initial window height
        #[arg(long)]
        height: Option<f64>,

        /// Inject custom JavaScript (file path or code)
        #[arg(long)]
        inject_js: Option<String>,

        /// Inject custom CSS stylesheet (file path or code)
        #[arg(long)]
        inject_css: Option<String>,
    },
    /// Manage curated built-in app presets (WhatsApp, Discord)
    Preset {
        #[command(subcommand)]
        action: PresetCommands,
    },
    /// Reconfigure settings of an installed app without wiping session data
    Configure {
        /// Target URL or alias of installed app
        url: String,

        /// Display Name
        #[arg(long)]
        name: Option<String>,

        /// Window class
        #[arg(long)]
        wm_class: Option<String>,

        /// Additional allowed domains
        #[arg(short = 'a', long = "allow-domain")]
        allow_domain: Vec<String>,

        /// Keep app running in background when closed
        #[arg(long)]
        hide_on_close: Option<bool>,

        /// Enforce single instance
        #[arg(long)]
        single_instance: Option<bool>,

        /// System tray icon
        #[arg(long)]
        tray: Option<bool>,

        /// Launch hidden in background
        #[arg(long)]
        start_hidden: Option<bool>,

        /// Initial zoom level
        #[arg(long)]
        zoom: Option<f64>,

        /// Override User-Agent
        #[arg(long)]
        user_agent: Option<String>,

        /// Window width
        #[arg(long)]
        width: Option<f64>,

        /// Window height
        #[arg(long)]
        height: Option<f64>,

        /// Autostart on boot
        #[arg(long)]
        autostart: Option<bool>,

        /// Autostart hidden in tray on boot
        #[arg(long)]
        autostart_hidden: Option<bool>,

        /// Inject custom JavaScript
        #[arg(long)]
        inject_js: Option<String>,

        /// Inject custom CSS
        #[arg(long)]
        inject_css: Option<String>,
    },
    /// Reinstall an app, re-fetching icon and updating launcher
    Reinstall {
        /// Target URL or alias
        url: String,
    },
    /// Clear cookies and session cache for an app
    ClearCache {
        /// Target URL or alias
        url: String,
    },
    /// Uninstall an app by URL or alias
    Uninstall {
        url: String,
    },
    /// List all installed applications
    List,
    /// Install the appify binary itself to ~/.local/bin and add desktop launcher
    SelfInstall,
    /// Launch the modern graphical Appify Manager
    Ui,
    /// Launch the modern graphical Appify Manager
    Gui,
    /// Browse and install plugins and themes from the community CDN
    Marketplace {
        #[command(subcommand)]
        action: MarketplaceCommands,
    },
}

#[derive(Subcommand)]
enum MarketplaceCommands {
    /// List available community plugins and themes on the CDN
    List,
    /// Search marketplace extensions by title, tags, or description
    Search {
        /// Search keyword
        query: String,
    },
    /// Install an extension from the CDN into the local registry
    Install {
        /// Unique extension ID (e.g. github-wide-diffs, oled-true-black)
        id: String,
    },
}

#[derive(Subcommand)]
enum PresetCommands {
    /// List all available built-in app presets
    List,
    /// Install a curated preset application (e.g. whatsapp, discord, telegram, spotify)
    Install {
        /// Preset name (whatsapp, discord, telegram, spotify)
        name: String,

        /// Custom display name override
        #[arg(long)]
        name_override: Option<String>,

        /// Custom WM_CLASS override
        #[arg(long)]
        wm_class: Option<String>,

        /// Custom icon path or URL
        #[arg(long)]
        icon: Option<String>,

        /// Additional domains to keep inside the app
        #[arg(short = 'a', long = "allow-domain")]
        allow_domain: Vec<String>,

        /// Keep app running in background when window is closed
        #[arg(long)]
        hide_on_close: bool,

        /// Enforce only one running instance of this app
        #[arg(long)]
        single_instance: bool,

        /// Show system tray icon
        #[arg(long)]
        tray: bool,

        /// Launch the window hidden in background / tray
        #[arg(long)]
        start_hidden: bool,

        /// Initial zoom level
        #[arg(long)]
        zoom: Option<f64>,

        /// Launch app automatically on user login
        #[arg(long)]
        autostart: bool,

        /// Launch app automatically on user login in background/tray
        #[arg(long)]
        autostart_hidden: bool,

        /// Inject custom JavaScript
        #[arg(long)]
        inject_js: Option<String>,

        /// Inject custom CSS
        #[arg(long)]
        inject_css: Option<String>,
    },
}

#[derive(Debug, Clone, Default)]
struct RunOptions {
    custom_name: Option<String>,
    custom_wm_class: Option<String>,
    custom_icon: Option<String>,
    cli_allowed_domains: Vec<String>,
    hide_on_close: bool,
    single_instance: bool,
    tray: bool,
    start_hidden: bool,
    maximize: bool,
    zoom: Option<f64>,
    user_agent: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    autostart: bool,
    autostart_hidden: bool,
    inject_js: Option<String>,
    inject_css: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String, // "plugin" (JS) or "theme" (CSS)
    pub content: String,
    pub author: String,
    pub version: String,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionRegistry {
    pub plugins: Vec<ExtensionItem>,
    pub themes: Vec<ExtensionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppExtensionRef {
    pub id: String,
    pub name: String,
    pub category: String, // "plugin" or "theme"
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub custom_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AppMetadata {
    url: String,
    base_domain: String,
    name: String,
    wm_class: String,
    custom_icon: Option<String>,
    hash: String,
    allowed_domains: Vec<String>,
    custom_allowed_domains: Vec<String>,
    hide_on_close: bool,
    single_instance: bool,
    tray: bool,
    start_hidden: bool,
    maximize: bool,
    zoom: Option<f64>,
    user_agent: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    autostart: bool,
    autostart_hidden: bool,
    #[serde(default)]
    inject_js: Option<String>,
    #[serde(default)]
    inject_css: Option<String>,
    #[serde(default)]
    app_plugins: Vec<AppExtensionRef>,
    #[serde(default)]
    app_themes: Vec<AppExtensionRef>,
}

#[allow(dead_code)]
type AppConfig = AppMetadata;

const WHATSAPP_ICON_PNG: &[u8] = include_bytes!("assets/whatsapp.png");
const DISCORD_ICON_PNG: &[u8] = include_bytes!("assets/discord.png");
const TELEGRAM_ICON_PNG: &[u8] = include_bytes!("assets/telegram.png");
const SPOTIFY_ICON_PNG: &[u8] = include_bytes!("assets/spotify.png");

#[derive(Debug, Clone, Serialize)]
struct Preset {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    category: &'static str,
    url: &'static str,
    wm_class: &'static str,
    default_hide_on_close: bool,
    default_single_instance: bool,
    default_tray: bool,
    default_autostart_hidden: bool,
    ecosystem_domains: &'static [&'static str],
    #[serde(skip)]
    default_inject_js: Option<&'static str>,
    #[serde(skip)]
    default_inject_css: Option<&'static str>,
    #[serde(skip)]
    bundled_icon: Option<&'static [u8]>,
}

fn get_curated_presets() -> Vec<Preset> {
    vec![
        Preset {
            id: "whatsapp",
            name: "WhatsApp",
            description: "Official WhatsApp Web client with system tray, background notifications, and clipboard image pasting.",
            category: "Messaging & Calling",
            url: "https://web.whatsapp.com",
            wm_class: "whatsapp-desktop",
            default_hide_on_close: true,
            default_single_instance: true,
            default_tray: true,
            default_autostart_hidden: false,
            ecosystem_domains: &[
                "whatsapp.com",
                "whatsapp.net",
                "fbcdn.net",
                "facebook.com",
                "messenger.com",
            ],
            default_inject_css: Some(r#"
/* Hide 'Get WhatsApp for Mac / Windows' desktop app promo banners */
a[href*="whatsapp.com/download"],
a[href*="microsoft.com/store"][href*="whatsapp"],
button[aria-label*="Get WhatsApp" i],
button[aria-label*="Get the app" i],
button[title*="Get WhatsApp" i],
div[aria-label*="Get WhatsApp" i],
span[aria-label*="Get WhatsApp" i],
div[data-testid*="intro-banner"],
div[data-testid*="native-desktop-banner"],
div[data-testid*="desktop-app-banner"],
div[data-testid*="get-desktop-app"] {
    display: none !important;
}
"#),
            default_inject_js: Some(r#"
(function() {
    function cleanWhatsAppDownloadPromos() {
        var selectors = [
            'a[href*="whatsapp.com/download"]',
            'a[href*="microsoft.com/store"][href*="whatsapp"]',
            'button[aria-label*="Get WhatsApp" i]',
            'button[aria-label*="Get the app" i]',
            'div[aria-label*="Get WhatsApp" i]',
            'span[aria-label*="Get WhatsApp" i]',
            'div[data-testid*="intro-banner"]',
            'div[data-testid*="native-desktop-banner"]',
            'div[data-testid*="desktop-app-banner"]',
            'div[data-testid*="get-desktop-app"]'
        ];
        var els = document.querySelectorAll(selectors.join(','));
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            var btn = el.closest('div[role="button"]') || el.closest('button') || el;
            btn.style.setProperty('display', 'none', 'important');
        }

        try {
            var walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
            var node;
            var toHide = [];
            while (node = walker.nextNode()) {
                var text = node.textContent.trim();
                if (/Get WhatsApp for (Mac|Windows)/i.test(text) || /^Get WhatsApp$/i.test(text)) {
                    var container = node.parentElement;
                    for (var k = 0; k < 5 && container && container !== document.body; k++) {
                        if (container.getAttribute('role') === 'button' || container.tagName === 'BUTTON' || container.tagName === 'A') {
                            break;
                        }
                        container = container.parentElement;
                    }
                    if (container) toHide.push(container);
                }
            }
            for (var j = 0; j < toHide.length; j++) {
                toHide[j].style.setProperty('display', 'none', 'important');
            }
        } catch (e) {}
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', cleanWhatsAppDownloadPromos);
    } else {
        cleanWhatsAppDownloadPromos();
    }

    var observer = new MutationObserver(cleanWhatsAppDownloadPromos);
    observer.observe(document.documentElement, { childList: true, subtree: true });
})();
"#),
            bundled_icon: Some(WHATSAPP_ICON_PNG),
        },
        Preset {
            id: "discord",
            name: "Discord",
            description: "Voice, video, and text communication platform with tray support, single-instance locking, and media streaming.",
            category: "Gaming & Community",
            url: "https://discord.com/app",
            wm_class: "discord-app",
            default_hide_on_close: true,
            default_single_instance: true,
            default_tray: true,
            default_autostart_hidden: false,
            ecosystem_domains: &[
                "discord.com",
                "discord.gg",
                "discordapp.com",
                "discordapp.net",
                "discord.media",
                "discordcdn.com",
            ],
            default_inject_css: None,
            default_inject_js: None,
            bundled_icon: Some(DISCORD_ICON_PNG),
        },
        Preset {
            id: "telegram",
            name: "Telegram",
            description: "Fast and secure cloud-based messaging client with system tray, background notifications, and media streaming.",
            category: "Messaging & Calling",
            url: "https://web.telegram.org",
            wm_class: "telegram-web",
            default_hide_on_close: true,
            default_single_instance: true,
            default_tray: true,
            default_autostart_hidden: false,
            ecosystem_domains: &[
                "telegram.org",
                "t.me",
                "web.telegram.org",
                "telesco.pe",
                "telegram.me",
            ],
            default_inject_css: None,
            default_inject_js: None,
            bundled_icon: Some(TELEGRAM_ICON_PNG),
        },
        Preset {
            id: "spotify",
            name: "Spotify",
            description: "Digital music, podcast, and audio streaming service with tray controls, background playback, and media keys.",
            category: "Media & Audio",
            url: "https://open.spotify.com",
            wm_class: "spotify-web",
            default_hide_on_close: true,
            default_single_instance: true,
            default_tray: true,
            default_autostart_hidden: false,
            ecosystem_domains: &[
                "spotify.com",
                "scdn.co",
                "spotifycdn.com",
                "spotify.link",
                "audio-ak-spotify-com.akamaized.net",
            ],
            default_inject_css: None,
            default_inject_js: None,
            bundled_icon: Some(SPOTIFY_ICON_PNG),
        },
    ]
}

#[derive(Debug, Clone)]
struct AliasInfo {
    url: &'static str,
    name: &'static str,
    wm_class: &'static str,
    ecosystem_domains: &'static [&'static str],
}


fn get_popular_aliases() -> HashMap<&'static str, AliasInfo> {
    let mut map = HashMap::new();
    map.insert(
        "whatsapp",
        AliasInfo {
            url: "https://web.whatsapp.com",
            name: "WhatsApp",
            wm_class: "whatsapp-desktop",
            ecosystem_domains: &[
                "whatsapp.com",
                "whatsapp.net",
                "fbcdn.net",
                "facebook.com",
                "messenger.com",
            ],
        },
    );
    map.insert(
        "discord",
        AliasInfo {
            url: "https://discord.com/app",
            name: "Discord",
            wm_class: "discord-app",
            ecosystem_domains: &[
                "discord.com",
                "discord.gg",
                "discordapp.com",
                "discordapp.net",
                "discord.media",
                "discordcdn.com",
            ],
        },
    );
    map.insert(
        "telegram",
        AliasInfo {
            url: "https://web.telegram.org",
            name: "Telegram",
            wm_class: "telegram-web",
            ecosystem_domains: &[
                "telegram.org",
                "t.me",
                "web.telegram.org",
                "telesco.pe",
            ],
        },
    );
    map.insert(
        "spotify",
        AliasInfo {
            url: "https://open.spotify.com",
            name: "Spotify",
            wm_class: "spotify-web",
            ecosystem_domains: &[
                "spotify.com",
                "scdn.co",
                "spotifycdn.com",
                "spotify.link",
            ],
        },
    );
    map.insert(
        "netflix",
        AliasInfo {
            url: "https://www.netflix.com",
            name: "Netflix",
            wm_class: "netflix-app",
            ecosystem_domains: &[
                "netflix.com",
                "nflxext.com",
                "nflximg.net",
                "nflxso.net",
                "nflxvideo.net",
            ],
        },
    );
    map.insert(
        "youtube",
        AliasInfo {
            url: "https://youtube.com",
            name: "YouTube",
            wm_class: "youtube-app",
            ecosystem_domains: &[
                "youtube.com",
                "youtu.be",
                "googlevideo.com",
                "ytimg.com",
                "google.com",
                "gstatic.com",
                "accounts.google.com",
            ],
        },
    );
    map.insert(
        "twitter",
        AliasInfo {
            url: "https://x.com",
            name: "X",
            wm_class: "twitter-x",
            ecosystem_domains: &["x.com", "twitter.com", "t.co", "twimg.com"],
        },
    );
    map.insert(
        "x",
        AliasInfo {
            url: "https://x.com",
            name: "X",
            wm_class: "twitter-x",
            ecosystem_domains: &["x.com", "twitter.com", "t.co", "twimg.com"],
        },
    );
    map.insert(
        "reddit",
        AliasInfo {
            url: "https://reddit.com",
            name: "Reddit",
            wm_class: "reddit-app",
            ecosystem_domains: &[
                "reddit.com",
                "redd.it",
                "redditstatic.com",
                "redditmedia.com",
            ],
        },
    );
    map.insert(
        "chatgpt",
        AliasInfo {
            url: "https://chatgpt.com",
            name: "ChatGPT",
            wm_class: "chatgpt-app",
            ecosystem_domains: &[
                "chatgpt.com",
                "openai.com",
                "oaistatic.com",
                "oaiusercontent.com",
                "auth0.com",
            ],
        },
    );
    map.insert(
        "notion",
        AliasInfo {
            url: "https://notion.so",
            name: "Notion",
            wm_class: "notion-app",
            ecosystem_domains: &["notion.so", "notion.site", "notion.com"],
        },
    );
    map.insert(
        "figma",
        AliasInfo {
            url: "https://figma.com",
            name: "Figma",
            wm_class: "figma-app",
            ecosystem_domains: &["figma.com"],
        },
    );
    map.insert(
        "gmail",
        AliasInfo {
            url: "https://mail.google.com",
            name: "Gmail",
            wm_class: "gmail-app",
            ecosystem_domains: &[
                "mail.google.com",
                "google.com",
                "accounts.google.com",
                "gstatic.com",
                "googleusercontent.com",
            ],
        },
    );
    map
}

fn get_ecosystem_domains_for_base(base_domain: &str) -> Vec<&'static str> {
    match base_domain {
        "whatsapp.com" | "whatsapp.net" => vec![
            "whatsapp.com",
            "whatsapp.net",
            "fbcdn.net",
            "facebook.com",
            "messenger.com",
        ],
        "discord.com" | "discord.gg" | "discordapp.com" => vec![
            "discord.com",
            "discord.gg",
            "discordapp.com",
            "discordapp.net",
            "discord.media",
            "discordcdn.com",
        ],
        "telegram.org" | "t.me" => {
            vec!["telegram.org", "t.me", "web.telegram.org", "telesco.pe"]
        }
        "spotify.com" => vec!["spotify.com", "scdn.co", "spotifycdn.com", "spotify.link"],
        "netflix.com" => vec![
            "netflix.com",
            "nflxext.com",
            "nflximg.net",
            "nflxso.net",
            "nflxvideo.net",
        ],
        "youtube.com" | "youtu.be" => vec![
            "youtube.com",
            "youtu.be",
            "googlevideo.com",
            "ytimg.com",
            "google.com",
            "gstatic.com",
            "accounts.google.com",
        ],
        "x.com" | "twitter.com" => vec!["x.com", "twitter.com", "t.co", "twimg.com"],
        "reddit.com" | "redd.it" => vec![
            "reddit.com",
            "redd.it",
            "redditstatic.com",
            "redditmedia.com",
        ],
        "openai.com" | "chatgpt.com" => vec![
            "chatgpt.com",
            "openai.com",
            "oaistatic.com",
            "oaiusercontent.com",
            "auth0.com",
        ],
        "notion.so" | "notion.site" => vec!["notion.so", "notion.site", "notion.com"],
        "gmail.com" => vec![
            "mail.google.com",
            "google.com",
            "accounts.google.com",
            "gstatic.com",
            "googleusercontent.com",
        ],
        _ => vec![],
    }
}

const COMMON_SSO_DOMAINS: &[&str] = &[
    "accounts.google.com",
    "appleid.apple.com",
    "login.microsoftonline.com",
    "login.live.com",
];

const BROWSER_INTEGRATION_SCRIPT: &str = r#"
(function() {
    // 1. Notification API polyfill
    try {
        if (typeof window.Notification === 'undefined' || window.Notification.permission !== 'granted') {
            function AppifyNotification(title, options) {
                this.title = title;
                this.options = options || {};
                this.onclick = null;
                this.onclose = null;
                this.onerror = null;
                this.onshow = null;
            }
            AppifyNotification.permission = 'granted';
            AppifyNotification.requestPermission = function(cb) {
                var promise = Promise.resolve('granted');
                if (typeof cb === 'function') {
                    promise.then(cb);
                }
                return promise;
            };
            AppifyNotification.prototype.close = function() {
                if (typeof this.onclose === 'function') {
                    this.onclose();
                }
            };
            AppifyNotification.prototype.addEventListener = function(type, listener) {
                this['on' + type] = listener;
            };
            AppifyNotification.prototype.removeEventListener = function(type) {
                this['on' + type] = null;
            };
            AppifyNotification.prototype.dispatchEvent = function() {
                return true;
            };
            window.Notification = AppifyNotification;
        }
    } catch (e) {}

    // 2. Permissions API polyfill / interceptor (auto-grant clipboard, mic, camera, notifications)
    try {
        if (!navigator.permissions) {
            navigator.permissions = {};
        }
        var origQuery = navigator.permissions.query ? navigator.permissions.query.bind(navigator.permissions) : null;
        navigator.permissions.query = function(desc) {
            var autoAllow = [
                'clipboard-read',
                'clipboard-write',
                'notifications',
                'microphone',
                'camera',
                'persistent-storage'
            ];
            var name = desc && desc.name ? desc.name : '';
            if (autoAllow.indexOf(name) !== -1) {
                return Promise.resolve({
                    state: 'granted',
                    name: name,
                    onchange: null,
                    addEventListener: function() {},
                    removeEventListener: function() {},
                    dispatchEvent: function() { return true; }
                });
            }
            if (origQuery) {
                return origQuery(desc).catch(function() {
                    return {
                        state: 'granted',
                        name: name || 'unknown',
                        onchange: null,
                        addEventListener: function() {},
                        removeEventListener: function() {},
                        dispatchEvent: function() { return true; }
                    };
                });
            }
            return Promise.resolve({
                state: 'granted',
                name: name || 'unknown',
                onchange: null,
                addEventListener: function() {},
                removeEventListener: function() {},
                dispatchEvent: function() { return true; }
            });
        };
    } catch (e) {}

    // 3. Fallback for navigator.clipboard.read if missing / image support
    window.__appify_latest_image_blob = null;
    try {
        if (navigator.clipboard) {
            var origRead = navigator.clipboard.read ? navigator.clipboard.read.bind(navigator.clipboard) : null;
            navigator.clipboard.read = function() {
                if (window.__appify_latest_image_blob && typeof ClipboardItem !== 'undefined') {
                    return Promise.resolve([
                        new ClipboardItem({ 'image/png': window.__appify_latest_image_blob })
                    ]);
                }
                if (origRead) {
                    return origRead().catch(function() {
                        if (navigator.clipboard.readText) {
                            return navigator.clipboard.readText().then(function(text) {
                                if (typeof ClipboardItem !== 'undefined') {
                                    var blob = new Blob([text], { type: 'text/plain' });
                                    return [new ClipboardItem({ 'text/plain': blob })];
                                }
                                return [];
                            });
                        }
                        return [];
                    });
                }
                if (navigator.clipboard.readText) {
                    return navigator.clipboard.readText().then(function(text) {
                        if (typeof ClipboardItem !== 'undefined') {
                            var blob = new Blob([text], { type: 'text/plain' });
                            return [new ClipboardItem({ 'text/plain': blob })];
                        }
                        return [];
                    });
                }
                return Promise.resolve([]);
            };
        }
    } catch (e) {}

    // 4. Injected image cache for paste and drop
    window.__appify_clipboard_image = null;

    function __appify_create_image_datatransfer(base64Data, fileName, mimeType) {
        fileName = fileName || 'pasted_image.png';
        mimeType = mimeType || 'image/png';
        var byteChars = atob(base64Data);
        var byteNums = new Array(byteChars.length);
        for (var i = 0; i < byteChars.length; i++) {
            byteNums[i] = byteChars.charCodeAt(i);
        }
        var byteArray = new Uint8Array(byteNums);
        var blob = new Blob([byteArray], { type: mimeType });
        var file = new File([blob], fileName, {
            type: mimeType,
            lastModified: Date.now()
        });
        var dt = new DataTransfer();
        dt.items.add(file);
        return { dt: dt, file: file, blob: blob };
    }

    // Dispatch dropped files into WhatsApp / web app
    window.__appify_handle_dropped_files = function(files) {
        if (!files || !files.length) return;
        try {
            var dt = new DataTransfer();
            for (var i = 0; i < files.length; i++) {
                var f = files[i];
                var byteChars = atob(f.data);
                var byteNums = new Array(byteChars.length);
                for (var j = 0; j < byteChars.length; j++) {
                    byteNums[j] = byteChars.charCodeAt(j);
                }
                var blob = new Blob([new Uint8Array(byteNums)], { type: f.type || 'image/png' });
                var file = new File([blob], f.name || 'dropped_image.png', {
                    type: f.type || 'image/png',
                    lastModified: Date.now()
                });
                dt.items.add(file);
            }

            // Route 1: Target file input if available (WhatsApp / Discord / etc.)
            var fileInputs = document.querySelectorAll('input[type="file"]');
            for (var k = 0; k < fileInputs.length; k++) {
                var acc = fileInputs[k].getAttribute('accept') || '';
                if (acc.indexOf('image') !== -1 || acc === '*' || acc === '') {
                    fileInputs[k].files = dt.files;
                    fileInputs[k].dispatchEvent(new Event('change', { bubbles: true }));
                    return;
                }
            }

            // Route 2: Try mounting attachment input if unmounted
            var attachBtn = document.querySelector('button[aria-label="Attach"]') ||
                            document.querySelector('div[aria-label="Attach"]') ||
                            document.querySelector('span[data-icon="plus"]') ||
                            document.querySelector('span[data-icon="attach-menu-plus"]');
            if (attachBtn) {
                attachBtn.click();
                setTimeout(function() {
                    var inputs = document.querySelectorAll('input[type="file"]');
                    for (var m = 0; m < inputs.length; m++) {
                        var a = inputs[m].getAttribute('accept') || '';
                        if (a.indexOf('image') !== -1 || a === '*' || a === '') {
                            inputs[m].files = dt.files;
                            inputs[m].dispatchEvent(new Event('change', { bubbles: true }));
                            return;
                        }
                    }
                }, 80);
            }

            // Route 3: Dispatch native drop event
            var dropTarget = document.querySelector('#main') ||
                             document.querySelector('div[data-testid="conversation-panel-wrapper"]') ||
                             document.activeElement ||
                             document.body;
            var dropEvent = new DragEvent('drop', {
                bubbles: true,
                cancelable: true,
                composed: true,
                dataTransfer: dt
            });
            dropTarget.dispatchEvent(dropEvent);
        } catch (err) {
            console.error('[Appify] Failed to handle dropped files:', err);
        }
    };

    // Synthetic paste event dispatcher for images
    window.__appify_dispatch_paste_image = function(base64Data) {
        try {
            var res = __appify_create_image_datatransfer(base64Data, 'pasted_image.png', 'image/png');
            var dt = res.dt;
            window.__appify_latest_image_blob = res.blob;

            var pasteEvent;
            try {
                pasteEvent = new ClipboardEvent('paste', {
                    bubbles: true,
                    cancelable: true,
                    composed: true,
                    clipboardData: dt
                });
            } catch (err) {
                pasteEvent = document.createEvent('Event');
                pasteEvent.initEvent('paste', true, true);
                pasteEvent.clipboardData = dt;
            }
            pasteEvent.__appify_synthetic = true;

            var target = document.activeElement;
            if (!target || target === document.body || target === document.documentElement) {
                target = document.querySelector('[contenteditable="true"]') ||
                         document.querySelector('div[role="textbox"]') ||
                         document.querySelector('input') ||
                         document.querySelector('textarea') ||
                         document.body ||
                         document;
            }
            target.dispatchEvent(pasteEvent);
        } catch (e) {
            console.error('[Appify] Failed to dispatch paste image:', e);
        }
    };

    // 5. Intercept native paste events (Capture Phase)
    window.addEventListener('paste', function(e) {
        if (e.__appify_synthetic) return;

        if (window.__appify_clipboard_image) {
            try {
                var res = __appify_create_image_datatransfer(window.__appify_clipboard_image, 'pasted_image.png', 'image/png');
                var dt = res.dt;

                if (e.clipboardData && e.clipboardData.items) {
                    for (var j = 0; j < e.clipboardData.items.length; j++) {
                        var it = e.clipboardData.items[j];
                        if (it.type && it.type.indexOf('image') === -1) {
                            try {
                                var itText = e.clipboardData.getData(it.type);
                                if (itText) dt.setData(it.type, itText);
                            } catch (err) {}
                        }
                    }
                }

                try {
                    Object.defineProperty(e, 'clipboardData', {
                        get: function() { return dt; },
                        configurable: true
                    });
                } catch (err) {}

                setTimeout(function() {
                    var fileInputs = document.querySelectorAll('input[type="file"]');
                    for (var k = 0; k < fileInputs.length; k++) {
                        var acc = fileInputs[k].getAttribute('accept') || '';
                        if (acc.indexOf('image') !== -1) {
                            fileInputs[k].files = dt.files;
                            fileInputs[k].dispatchEvent(new Event('change', { bubbles: true }));
                            break;
                        }
                    }
                }, 70);
            } catch (err) {
                console.error('[Appify] Error during paste event decoration:', err);
            }
            return;
        }

        var hasImage = false;
        if (e.clipboardData && e.clipboardData.items) {
            for (var i = 0; i < e.clipboardData.items.length; i++) {
                if (e.clipboardData.items[i].type && e.clipboardData.items[i].type.indexOf('image') !== -1) {
                    hasImage = true;
                    break;
                }
            }
        }
        if (!hasImage && window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.appify) {
            window.webkit.messageHandlers.appify.postMessage('check_clipboard_paste');
        }
    }, true);

    // 6. DevTools keyboard shortcuts (F12, Ctrl+Shift+I, Ctrl+Shift+C, Ctrl+Shift+J)
    window.addEventListener('keydown', function(e) {
        var isCtrlOrCmd = e.ctrlKey || e.metaKey;
        var isShift = e.shiftKey;
        var key = e.key ? e.key.toUpperCase() : '';

        var isDevTools = (key === 'F12') || (isCtrlOrCmd && isShift && (key === 'I' || key === 'J' || key === 'C'));
        if (isDevTools) {
            e.preventDefault();
            e.stopPropagation();
            if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.appify) {
                window.webkit.messageHandlers.appify.postMessage('toggle_devtools');
            }
        }
    }, true);

    // 7. Intercept downloads (blob:, data:, and anchor downloads)
    var __appify_last_dl_url = '';
    var __appify_last_dl_time = 0;
    window.__appify_active_blobs = window.__appify_active_blobs || {};

    function __appify_mime_to_ext(mime) {
        if (!mime) return '';
        mime = mime.toLowerCase();
        if (mime.indexOf('image/jpeg') !== -1) return 'jpg';
        if (mime.indexOf('image/png') !== -1) return 'png';
        if (mime.indexOf('image/webp') !== -1) return 'webp';
        if (mime.indexOf('image/gif') !== -1) return 'gif';
        if (mime.indexOf('video/mp4') !== -1) return 'mp4';
        if (mime.indexOf('video/webm') !== -1) return 'webm';
        if (mime.indexOf('audio/ogg') !== -1 || mime.indexOf('audio/opus') !== -1) return 'ogg';
        if (mime.indexOf('audio/mpeg') !== -1 || mime.indexOf('audio/mp3') !== -1) return 'mp3';
        if (mime.indexOf('audio/mp4') !== -1 || mime.indexOf('audio/m4a') !== -1) return 'm4a';
        if (mime.indexOf('audio/aac') !== -1) return 'aac';
        if (mime.indexOf('application/pdf') !== -1) return 'pdf';
        if (mime.indexOf('application/zip') !== -1) return 'zip';
        if (mime.indexOf('text/plain') !== -1) return 'txt';
        return '';
    }

    function __appify_handle_download_url(url, suggestedFilename) {
        if (!url || typeof url !== 'string') return false;
        var isBlob = url.startsWith('blob:');
        var isData = url.startsWith('data:');
        if (!isBlob && !isData) {
            return false;
        }

        var now = Date.now();
        if (url === __appify_last_dl_url && (now - __appify_last_dl_time) < 1000) {
            return true;
        }
        __appify_last_dl_url = url;
        __appify_last_dl_time = now;

        var filename = suggestedFilename ? suggestedFilename.trim() : '';

        fetch(url)
            .then(function(res) {
                var mime = res.headers.get('content-type') || '';
                return res.blob().then(function(blob) {
                    return { blob: blob, mime: mime || blob.type };
                });
            })
            .then(function(res) {
                var blob = res.blob;
                var mime = res.mime || blob.type || '';
                if (!filename || filename.indexOf('.') === -1) {
                    var ext = __appify_mime_to_ext(mime) || 'bin';
                    if (!filename) {
                        filename = 'download.' + ext;
                    } else if (filename.indexOf('.') === -1) {
                        filename = filename + '.' + ext;
                    }
                }

                var transferId = 'dl_' + Date.now() + '_' + Math.random().toString(36).substring(2, 8);
                window.__appify_active_blobs[transferId] = {
                    blob: blob,
                    filename: filename,
                    mime: mime
                };

                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.appify) {
                    window.webkit.messageHandlers.appify.postMessage(JSON.stringify({
                        type: 'download_request',
                        transferId: transferId,
                        filename: filename,
                        size: blob.size,
                        mime: mime
                    }));
                }
            })
            .catch(function(err) {
                console.error('[Appify] Failed to fetch blob for download:', err);
            });

        return true;
    }

    // Anchor prototype click hook
    try {
        var origAnchorClick = HTMLAnchorElement.prototype.click;
        HTMLAnchorElement.prototype.click = function() {
            var href = this.href || '';
            var hasDownload = this.hasAttribute('download');
            if (hasDownload || href.startsWith('blob:') || href.startsWith('data:')) {
                var filename = this.getAttribute('download') || this.download || '';
                if (__appify_handle_download_url(href, filename)) {
                    return;
                }
            }
            return origAnchorClick.apply(this, arguments);
        };
    } catch (e) {}

    // Global capture-phase click hook for anchors
    window.addEventListener('click', function(e) {
        var el = e.target;
        var anchor = el && el.closest ? el.closest('a') : null;
        if (anchor) {
            var href = anchor.href || '';
            var hasDownload = anchor.hasAttribute('download');
            if (hasDownload || href.startsWith('blob:') || href.startsWith('data:')) {
                var filename = anchor.getAttribute('download') || anchor.download || '';
                if (__appify_handle_download_url(href, filename)) {
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                }
            }
        }
    }, true);

    // Window.open hook
    try {
        var origOpen = window.open;
        window.open = function(url) {
            if (typeof url === 'string' && (url.startsWith('blob:') || url.startsWith('data:'))) {
                if (__appify_handle_download_url(url, '')) {
                    return null;
                }
            }
            return origOpen.apply(this, arguments);
        };
    } catch (e) {}

    // Callback to begin chunk streaming once user confirms save location
    window.__appify_download_start_transfer = function(transferId) {
        var item = window.__appify_active_blobs && window.__appify_active_blobs[transferId];
        if (!item) return;
        var blob = item.blob;
        var CHUNK_SIZE = 1024 * 1024; // 1 MB chunk
        var totalSize = blob.size;
        var totalChunks = Math.max(1, Math.ceil(totalSize / CHUNK_SIZE));
        var currentChunk = 0;

        function sendNext() {
            if (currentChunk >= totalChunks) {
                delete window.__appify_active_blobs[transferId];
                return;
            }
            var start = currentChunk * CHUNK_SIZE;
            var end = Math.min(start + CHUNK_SIZE, totalSize);
            var slice = blob.slice(start, end);

            var reader = new FileReader();
            reader.onloadend = function() {
                var dataUrl = reader.result;
                var comma = dataUrl ? dataUrl.indexOf(',') : -1;
                var base64 = comma !== -1 ? dataUrl.substring(comma + 1) : '';
                var isLast = (currentChunk === totalChunks - 1);

                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.appify) {
                    window.webkit.messageHandlers.appify.postMessage(JSON.stringify({
                        type: 'download_chunk',
                        transferId: transferId,
                        chunkIndex: currentChunk,
                        totalChunks: totalChunks,
                        data: base64,
                        isLast: isLast
                    }));
                }
                currentChunk++;
                if (!isLast) {
                    setTimeout(sendNext, 5);
                } else {
                    delete window.__appify_active_blobs[transferId];
                }
            };
            reader.readAsDataURL(slice);
        }

        sendNext();
    };

    // Callback to cancel download if user aborted save dialog
    window.__appify_download_cancel = function(transferId) {
        if (window.__appify_active_blobs && window.__appify_active_blobs[transferId]) {
            delete window.__appify_active_blobs[transferId];
        }
    };

    // 8. Fix WebKit SVG calc() attribute parsing bug (e.g. calc(50% - 0px), calc(100% - 0px))
    try {
        var origSetAttribute = Element.prototype.setAttribute;
        var origSetAttributeNS = Element.prototype.setAttributeNS;

        function sanitizeSvgLength(val) {
            if (typeof val === 'string' && val.indexOf('calc(') !== -1) {
                var cleaned = val.replace(/calc\(\s*([\d.]+%?)\s*[+-]\s*0(?:px|%|\w+)?\s*\)/gi, '$1');
                if (cleaned.indexOf('calc(') !== -1) {
                    var m = cleaned.match(/calc\(\s*([\d.]+%?)/i);
                    if (m && m[1]) return m[1];
                }
                return cleaned;
            }
            return val;
        }

        Element.prototype.setAttribute = function(name, val) {
            if (this.namespaceURI === 'http://www.w3.org/2000/svg' || (this.tagName && (this.tagName.toLowerCase() === 'circle' || this.tagName.toLowerCase() === 'rect'))) {
                if (name === 'r' || name === 'width' || name === 'height' || name === 'x' || name === 'y' || name === 'cx' || name === 'cy') {
                    val = sanitizeSvgLength(val);
                }
            }
            try {
                return origSetAttribute.call(this, name, val);
            } catch (err) {
                if (typeof val === 'string' && val.indexOf('calc(') !== -1) {
                    return origSetAttribute.call(this, name, val.replace(/calc\([^)]+\)/gi, '50%'));
                }
                throw err;
            }
        };

        Element.prototype.setAttributeNS = function(ns, name, val) {
            if (this.namespaceURI === 'http://www.w3.org/2000/svg' || (this.tagName && (this.tagName.toLowerCase() === 'circle' || this.tagName.toLowerCase() === 'rect'))) {
                if (name === 'r' || name === 'width' || name === 'height' || name === 'x' || name === 'y' || name === 'cx' || name === 'cy') {
                    val = sanitizeSvgLength(val);
                }
            }
            try {
                return origSetAttributeNS.call(this, ns, name, val);
            } catch (err) {
                if (typeof val === 'string' && val.indexOf('calc(') !== -1) {
                    return origSetAttributeNS.call(this, ns, name, val.replace(/calc\([^)]+\)/gi, '50%'));
                }
                throw err;
            }
        };
    } catch (e) {}
})();
"#;

fn domain_matches(host: &str, pattern: &str) -> bool {
    let host = host.trim().to_lowercase();
    let mut pattern = pattern.trim().to_lowercase();
    if let Some(stripped) = pattern.strip_prefix("*.") {
        pattern = stripped.to_string();
    }
    if let Some(stripped) = pattern.strip_prefix('.') {
        pattern = stripped.to_string();
    }

    if host == pattern {
        return true;
    }
    if host.ends_with(&format!(".{}", pattern)) {
        return true;
    }
    false
}

fn is_internal_navigation(url: &Url, base_domain: &str, allowed_domains: &[String]) -> bool {
    let scheme = url.scheme();
    if scheme == "blob" || scheme == "data" || scheme == "about" || scheme == "javascript" {
        return true;
    }
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let host = match url.host_str() {
        Some(h) => h,
        None => return false,
    };
    if domain_matches(host, base_domain) {
        return true;
    }
    for allowed in allowed_domains {
        if domain_matches(host, allowed) {
            return true;
        }
    }
    false
}

fn resolve_script_content(input: Option<String>) -> Option<String> {
    let raw = input?;
    let path = Path::new(&raw);
    if path.exists() && path.is_file() {
        fs::read_to_string(path).ok()
    } else {
        Some(raw)
    }
}

fn resolve_metadata(
    raw_input: &str,
    opts: RunOptions,
) -> Result<AppMetadata, String> {
    let presets = get_curated_presets();
    let aliases = get_popular_aliases();
    let lower_input = raw_input.trim().to_lowercase();

    let (mut raw_url, default_name, default_wm, alias_domains) =
        if let Some(p) = presets.iter().find(|p| p.id == lower_input.as_str()) {
            (
                p.url.to_string(),
                Some(p.name.to_string()),
                Some(p.wm_class.to_string()),
                p.ecosystem_domains.to_vec(),
            )
        } else if let Some(info) = aliases.get(lower_input.as_str()) {
            (
                info.url.to_string(),
                Some(info.name.to_string()),
                Some(info.wm_class.to_string()),
                info.ecosystem_domains.to_vec(),
            )
        } else {
            (raw_input.trim().to_string(), None, None, vec![])
        };

    if !raw_url.starts_with("http://") && !raw_url.starts_with("https://") {
        raw_url = format!("https://{}", raw_url);
    }

    if raw_url.ends_with('/') {
        raw_url.pop();
    }

    let parsed = Url::parse(&raw_url).map_err(|e| format!("Invalid URL: {}", e))?;
    let netloc = parsed.host_str().ok_or("URL must have a host")?.to_string();

    if !netloc.contains('.') && netloc != "localhost" {
        return Err(format!("'{}' does not appear to be a valid domain.", raw_input));
    }

    let parts: Vec<&str> = netloc.split('.').collect();
    let base_domain = if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        netloc.clone()
    };

    let mut hasher = Sha256::new();
    hasher.update(raw_url.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let final_name = opts.custom_name.or(default_name).unwrap_or_else(|| netloc.clone());
    let final_wm = opts.custom_wm_class
        .or(default_wm)
        .unwrap_or_else(|| format!("appify-{}", &hash[..12]));

    let mut allowed_set = std::collections::BTreeSet::new();
    allowed_set.insert(base_domain.clone());

    for d in alias_domains {
        allowed_set.insert(d.to_string());
    }
    for d in get_ecosystem_domains_for_base(&base_domain) {
        allowed_set.insert(d.to_string());
    }
    for d in COMMON_SSO_DOMAINS {
        allowed_set.insert(d.to_string());
    }
    for d in &opts.cli_allowed_domains {
        allowed_set.insert(d.trim().to_lowercase());
    }

    let allowed_domains: Vec<String> = allowed_set.into_iter().collect();
    let autostart_hidden = opts.autostart_hidden || (opts.autostart && opts.start_hidden);
    let autostart = opts.autostart || autostart_hidden;
    let tray = opts.tray || opts.hide_on_close || opts.start_hidden || autostart_hidden;

    let matched_preset = presets.iter().find(|p| p.id == lower_input.as_str() || p.url == raw_url);
    let inject_js = resolve_script_content(opts.inject_js)
        .or_else(|| matched_preset.and_then(|p| p.default_inject_js.map(|s| s.to_string())));
    let inject_css = resolve_script_content(opts.inject_css)
        .or_else(|| matched_preset.and_then(|p| p.default_inject_css.map(|s| s.to_string())));

    let mut app_plugins = Vec::new();
    let mut app_themes = Vec::new();
    if let Some(p) = matched_preset {
        if p.id == "whatsapp" {
            app_plugins.push(AppExtensionRef {
                id: "builtin-wa-promo-cleaner".to_string(),
                name: "WhatsApp Promo Cleaner".to_string(),
                category: "plugin".to_string(),
                enabled: true,
                custom_content: None,
            });
            app_themes.push(AppExtensionRef {
                id: "builtin-wa-promo-css".to_string(),
                name: "WhatsApp Clean Layout".to_string(),
                category: "theme".to_string(),
                enabled: true,
                custom_content: None,
            });
        }
    }

    Ok(AppMetadata {
        url: raw_url,
        base_domain,
        name: final_name,
        wm_class: final_wm,
        custom_icon: opts.custom_icon,
        hash,
        allowed_domains,
        custom_allowed_domains: opts.cli_allowed_domains,
        hide_on_close: opts.hide_on_close,
        single_instance: opts.single_instance,
        tray,
        start_hidden: opts.start_hidden,
        maximize: opts.maximize,
        zoom: opts.zoom,
        user_agent: opts.user_agent,
        width: opts.width,
        height: opts.height,
        autostart,
        autostart_hidden,
        inject_js,
        inject_css,
        app_plugins,
        app_themes,
    })
}

static FALLBACK_ICON_BYTES: &[u8] = include_bytes!("default_icon.png");

fn convert_to_png_scored(bytes: &[u8]) -> Option<(Vec<u8>, bool)> {
    let dyn_img = image::load_from_memory(bytes).ok()?;
    let (w, h) = (dyn_img.width(), dyn_img.height());
    if w == 0 || h == 0 {
        return None;
    }
    let ratio = w as f32 / h as f32;
    let is_square = ratio >= 0.75 && ratio <= 1.33;
    let cropped = if w != h {
        let side = w.min(h);
        let x = (w - side) / 2;
        let y = (h - side) / 2;
        dyn_img.crop_imm(x, y, side, side)
    } else {
        dyn_img
    };
    let mut png_buf = Vec::new();
    cropped
        .write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)
        .ok()?;
    Some((png_buf, is_square))
}

fn convert_to_png(bytes: &[u8]) -> Option<Vec<u8>> {
    convert_to_png_scored(bytes).map(|(b, _)| b)
}

#[cfg(target_os = "linux")]
fn get_clipboard_image_png() -> Option<Vec<u8>> {
    if !gtk::is_initialized() && gtk::init().is_err() {
        return None;
    }
    let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
    if let Some(pixbuf) = clipboard.wait_for_image() {
        if let Ok(bytes) = pixbuf.save_to_bufferv("png", &[]) {
            return Some(bytes);
        }
    }
    let uris = clipboard.wait_for_uris();
    for uri_str in uris {
        if let Ok(url) = url::Url::parse(uri_str.as_str()) {
            if url.scheme() == "file" {
                if let Ok(path) = url.to_file_path() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if ["png", "jpg", "jpeg", "webp", "gif", "bmp"].contains(&ext.as_str()) {
                        if let Ok(bytes) = fs::read(&path) {
                            if ext == "png" {
                                return Some(bytes);
                            } else if let Some(png_bytes) = convert_to_png(&bytes) {
                                return Some(png_bytes);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn get_clipboard_image_png() -> Option<Vec<u8>> {
    None
}

struct ActiveBlobDownload {
    destination: PathBuf,
    file: Option<fs::File>,
    received_bytes: usize,
    total_bytes: usize,
}

#[cfg(target_os = "linux")]
fn prompt_save_file_picker(
    parent: Option<&gtk::Window>,
    default_filename: &str,
) -> Option<PathBuf> {
    use gtk::prelude::*;
    if !gtk::is_initialized() && gtk::init().is_err() {
        return None;
    }
    let raw_name = Path::new(default_filename)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("download");
    let clean_filename = if raw_name.trim().is_empty() {
        "download"
    } else {
        raw_name.trim()
    };

    let chooser = gtk::FileChooserNative::new(
        Some("Save File"),
        parent,
        gtk::FileChooserAction::Save,
        Some("_Save"),
        Some("_Cancel"),
    );
    chooser.set_do_overwrite_confirmation(true);
    chooser.set_current_name(clean_filename);
    if let Some(dl_dir) = dirs::download_dir() {
        let _ = chooser.set_current_folder(&dl_dir);
    }
    let res = chooser.run();
    if res == gtk::ResponseType::Accept {
        chooser.filename()
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn prompt_save_file_picker(
    _parent: Option<&()>,
    default_filename: &str,
) -> Option<PathBuf> {
    let dl_dir = dirs::download_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Downloads"))
            .unwrap_or_else(|| PathBuf::from("."))
    });
    Some(dl_dir.join(default_filename))
}


fn fetch_icon(url_str: &str, icon_path: &Path) {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(7))
        .build();

    let mut candidate_urls: Vec<String> = Vec::new();

    let html_body = if let Ok(resp) = agent
        .get(url_str)
        .set(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .set(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .call()
    {
        resp.into_string().ok()
    } else {
        std::process::Command::new("curl")
            .arg("-s")
            .arg("-L")
            .arg("--max-time")
            .arg("5")
            .arg("-A")
            .arg("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .arg(url_str)
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    String::from_utf8(out.stdout).ok()
                } else {
                    None
                }
            })
    };

    let mut apple_icons = Vec::new();
    let mut standard_icons = Vec::new();
    let mut og_icons = Vec::new();

    if let Some(body) = html_body {
        let doc = scraper::Html::parse_document(&body);
        let link_sel = scraper::Selector::parse(
            "link[rel*='icon'], link[rel*='apple-touch-icon'], meta[property='og:image']",
        )
        .unwrap();

        for el in doc.select(&link_sel) {
            let tag_name = el.value().name();
            if tag_name == "link" {
                if let Some(href) = el.value().attr("href") {
                    let rel = el.value().attr("rel").unwrap_or("").to_lowercase();
                    if rel.contains("apple-touch-icon") {
                        apple_icons.push(href.to_string());
                    } else if rel.contains("icon") {
                        standard_icons.push(href.to_string());
                    }
                }
            } else if tag_name == "meta" {
                if let Some(content) = el.value().attr("content") {
                    og_icons.push(content.to_string());
                }
            }
        }
    }

    // High priority: real application icons
    candidate_urls.extend(apple_icons);
    candidate_urls.extend(standard_icons);

    // Standard root paths
    if let Ok(base) = Url::parse(url_str) {
        if let Ok(fav) = base.join("/assets/favicon.ico") {
            candidate_urls.push(fav.to_string());
        }
        if let Ok(fav) = base.join("/favicon.ico") {
            candidate_urls.push(fav.to_string());
        }

        // If URL has a subpath, also check root origin
        if base.path() != "/" && !base.path().is_empty() {
            if let Ok(root) = base.join("/") {
                if let Ok(fav) = root.join("/favicon.ico") {
                    candidate_urls.push(fav.to_string());
                }
                if let Ok(fav) = root.join("/assets/favicon.ico") {
                    candidate_urls.push(fav.to_string());
                }
            }
        }
    } else {
        candidate_urls.push(format!("{}/favicon.ico", url_str));
    }

    // Low priority fallback: OpenGraph social media banners (only if no real icon exists)
    candidate_urls.extend(og_icons);

    let base_url = Url::parse(url_str).ok();
    let mut best_square_png: Option<Vec<u8>> = None;
    let mut fallback_cropped_png: Option<Vec<u8>> = None;

    for candidate in candidate_urls {
        let full_url = if let Some(ref base) = base_url {
            base.join(&candidate).map(|u| u.to_string()).unwrap_or(candidate)
        } else {
            candidate
        };

        let resp = agent
            .get(&full_url)
            .set(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            )
            .call();

        let bytes = match resp {
            Ok(r) => {
                let mut b = Vec::new();
                if r.into_reader().read_to_end(&mut b).is_ok() && !b.is_empty() {
                    b
                } else {
                    continue;
                }
            }
            Err(_) => {
                if let Ok(output) = std::process::Command::new("curl")
                    .arg("-s")
                    .arg("-L")
                    .arg("--max-time")
                    .arg("5")
                    .arg("-A")
                    .arg("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
                    .arg(&full_url)
                    .output()
                {
                    if output.status.success() && !output.stdout.is_empty() {
                        output.stdout
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        };

        if let Some((png_bytes, is_square)) = convert_to_png_scored(&bytes) {
            if is_square {
                best_square_png = Some(png_bytes);
                break;
            } else if fallback_cropped_png.is_none() {
                fallback_cropped_png = Some(png_bytes);
            }
        }
    }

    let final_png = best_square_png
        .or(fallback_cropped_png)
        .unwrap_or_else(|| FALLBACK_ICON_BYTES.to_vec());

    let _ = fs::write(icon_path, final_png);
}

// Sets Linux process comm, GTK program name (for WM_CLASS), and desktop application name
#[cfg(target_os = "linux")]
fn set_linux_app_id(wm_class: &str, app_name: &str) {
    use std::ffi::CString;
    extern "C" {
        fn g_set_prgname(prgname: *const i8);
        fn g_set_application_name(app_name: *const i8);
        fn prctl(option: i32, arg2: *const i8, arg3: u64, arg4: u64, arg5: u64) -> i32;
    }
    const PR_SET_NAME: i32 = 15;

    if let Ok(c_str) = CString::new(wm_class) {
        unsafe {
            g_set_prgname(c_str.as_ptr());
            prctl(PR_SET_NAME, c_str.as_ptr(), 0, 0, 0);
        }
    }
    if let Ok(c_str) = CString::new(app_name) {
        unsafe {
            g_set_application_name(c_str.as_ptr());
        }
    }
}

fn get_single_instance_lock_path(hash: &str) -> PathBuf {
    #[cfg(unix)]
    {
        let prefix = if hash.len() >= 16 { &hash[..16] } else { hash };
        dirs::runtime_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(format!("appify-{}.sock", prefix))
    }
    #[cfg(not(unix))]
    {
        get_profile_dir(hash).join("instance.port")
    }
}

#[cfg(unix)]
mod single_instance {
    use std::fs;
    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::Path;
    use tauri::{AppHandle, Manager};

    pub fn notify_existing(socket_path: &Path) -> bool {
        if let Ok(mut stream) = UnixStream::connect(socket_path) {
            let _ = stream.write_all(b"focus");
            return true;
        }
        false
    }

    pub fn start_listener(socket_path: &Path, handle: AppHandle) -> bool {
        let _ = fs::remove_file(socket_path);
        match UnixListener::bind(socket_path) {
            Ok(listener) => {
                let socket_path_buf = socket_path.to_path_buf();
                std::thread::spawn(move || {
                    for stream in listener.incoming() {
                        if let Ok(mut stream) = stream {
                            let mut buf = [0u8; 16];
                            if let Ok(n) = stream.read(&mut buf) {
                                if &buf[..n] == b"focus" {
                                    if let Some(w) = handle.get_webview_window("main") {
                                        let _ = w.show();
                                        let _ = w.unminimize();
                                        let _ = w.set_focus();
                                    }
                                }
                            }
                        }
                    }
                    let _ = fs::remove_file(&socket_path_buf);
                });
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(not(unix))]
mod single_instance {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::Path;
    use tauri::{AppHandle, Manager};

    pub fn notify_existing(port_file: &Path) -> bool {
        if let Ok(port_str) = fs::read_to_string(port_file) {
            if let Ok(port) = port_str.trim().parse::<u16>() {
                if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
                    let _ = stream.write_all(b"focus");
                    return true;
                }
            }
        }
        false
    }

    pub fn start_listener(port_file: &Path, handle: AppHandle) -> bool {
        match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => {
                if let Ok(addr) = listener.local_addr() {
                    let _ = fs::write(port_file, addr.port().to_string());
                    let port_file_buf = port_file.to_path_buf();
                    std::thread::spawn(move || {
                        for stream in listener.incoming() {
                            if let Ok(mut stream) = stream {
                                let mut buf = [0u8; 16];
                                if let Ok(n) = stream.read(&mut buf) {
                                    if &buf[..n] == b"focus" {
                                        if let Some(w) = handle.get_webview_window("main") {
                                            let _ = w.show();
                                            let _ = w.unminimize();
                                            let _ = w.set_focus();
                                        }
                                    }
                                }
                            }
                        }
                        let _ = fs::remove_file(&port_file_buf);
                    });
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

fn execute_run(meta: AppMetadata) {
    let target_url: Url = meta.url.parse().expect("Failed to parse URL");
    let base_domain = meta.base_domain.clone();
    let allowed_domains = meta.allowed_domains.clone();
    let win_title = meta.name.clone();

    #[cfg(target_os = "linux")]
    {
        set_linux_app_id(&meta.wm_class, &meta.name);
    }

    let data_dir = get_profile_dir(&meta.hash);

    fs::create_dir_all(&data_dir).ok();

    let mut effective_js = String::new();
    let js_file = data_dir.join("userscript.js");
    if js_file.exists() {
        if let Ok(content) = fs::read_to_string(&js_file) {
            effective_js.push_str(&content);
        }
    }
    if let Some(ref extra_js) = meta.inject_js {
        if !extra_js.trim().is_empty() {
            effective_js.push('\n');
            effective_js.push_str(extra_js);
        }
    }

    let mut effective_css = String::new();
    let css_file = data_dir.join("userstyle.css");
    if css_file.exists() {
        if let Ok(content) = fs::read_to_string(&css_file) {
            effective_css.push_str(&content);
        }
    }
    if let Some(ref extra_css) = meta.inject_css {
        if !extra_css.trim().is_empty() {
            effective_css.push('\n');
            effective_css.push_str(extra_css);
        }
    }

    let mut custom_init_script = String::new();
    if !effective_css.trim().is_empty() {
        let escaped_css = effective_css
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace('$', "\\$");
        custom_init_script.push_str(&format!(
            r#"
(function() {{
    const css = `{}`;
    function __appify_apply_css() {{
        if (document.getElementById('__appify_custom_style')) return;
        const s = document.createElement('style');
        s.id = '__appify_custom_style';
        s.textContent = css;
        (document.head || document.documentElement).appendChild(s);
    }}
    if (document.readyState === 'loading') {{
        document.addEventListener('DOMContentLoaded', __appify_apply_css);
    }} else {{
        __appify_apply_css();
    }}
}})();
"#,
            escaped_css
        ));
    }

    if !effective_js.trim().is_empty() {
        custom_init_script.push_str("\n(function() {\n");
        custom_init_script.push_str(&effective_js);
        custom_init_script.push_str("\n})();\n");
    }

    #[cfg(target_os = "linux")]
    let effective_css_for_linux = effective_css.clone();

    if meta.single_instance {
        let lock_path = get_single_instance_lock_path(&meta.hash);

        if single_instance::notify_existing(&lock_path) {
            println!("[+] '{}' is already running. Focused existing window.", meta.name);
            return;
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            if meta.single_instance {
                let lock_path = get_single_instance_lock_path(&meta.hash);
                let _ = single_instance::start_listener(&lock_path, app.handle().clone());
            }

            let handle_nav = app.handle().clone();
            let handle_new_win = app.handle().clone();

            let base_domain_for_nav = base_domain.clone();
            let allowed_domains_for_nav = allowed_domains.clone();

            let base_domain_for_new_win = base_domain.clone();
            let allowed_domains_for_new_win = allowed_domains.clone();

            let initial_width = meta.width.unwrap_or(1100.0);
            let initial_height = meta.height.unwrap_or(800.0);
            let start_visible = !meta.start_hidden;

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(target_url),
            )
                .title(&win_title)
                .inner_size(initial_width, initial_height)
                .min_inner_size(800.0, 600.0)
                .center()
                .visible(start_visible)
                .maximized(meta.maximize)
                .data_directory(data_dir)
                .enable_clipboard_access()
                .initialization_script(BROWSER_INTEGRATION_SCRIPT);

            if !custom_init_script.is_empty() {
                builder = builder.initialization_script(&custom_init_script);
            }

            if let Some(ref ua) = meta.user_agent {
                builder = builder.user_agent(ua);
            }

            if let Some(ref icon_path_str) = meta.custom_icon {
                let p = Path::new(icon_path_str);
                if p.exists() {
                    if let Ok(bytes) = fs::read(p) {
                        if let Ok(image) = tauri::image::Image::from_bytes(&bytes) {
                            builder = builder.icon(image)?;
                        }
                    }
                }
            }

            let win_title_for_doc = win_title.clone();
            let window = builder
                .on_navigation(move |nav_url| {
                    if is_internal_navigation(nav_url, &base_domain_for_nav, &allowed_domains_for_nav) {
                        true
                    } else {
                        let _ = handle_nav.opener().open_url(nav_url.as_str(), None::<&str>);
                        false
                    }
                })
                .on_new_window(move |new_url, _features| {
                    if is_internal_navigation(&new_url, &base_domain_for_new_win, &allowed_domains_for_new_win) {
                        tauri::webview::NewWindowResponse::Allow
                    } else {
                        let _ = handle_new_win.opener().open_url(new_url.as_str(), None::<&str>);
                        tauri::webview::NewWindowResponse::Deny
                    }
                })
                .on_document_title_changed(move |window, title| {
                    let effective_title = if title.trim().is_empty() {
                        win_title_for_doc.as_str()
                    } else {
                        title.as_str()
                    };
                    let _ = window.set_title(effective_title);

                    let is_unread = effective_title.starts_with('(') || effective_title.contains("Unread") || effective_title.contains("•");
                    if is_unread {
                        let _ = window.request_user_attention(Some(tauri::UserAttentionType::Informational));
                    } else {
                        let _ = window.request_user_attention(None);
                    }
                })
                .on_download(|_webview, event| {
                    match event {
                        tauri::webview::DownloadEvent::Requested { url, destination } => {
                            let filename = destination
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .filter(|s| !s.is_empty())
                                .or_else(|| {
                                    url.path_segments()
                                        .and_then(|mut segs| segs.next_back())
                                        .filter(|s| !s.is_empty())
                                        .map(|s| s.to_string())
                                })
                                .unwrap_or_else(|| "download".to_string());

                            #[cfg(target_os = "linux")]
                            {
                                if let Some(chosen_path) = prompt_save_file_picker(None, &filename) {
                                    *destination = chosen_path;
                                    true
                                } else {
                                    false
                                }
                            }
                            #[cfg(not(target_os = "linux"))]
                            {
                                let download_dir = dirs::download_dir().unwrap_or_else(|| {
                                    dirs::home_dir()
                                        .map(|h| h.join("Downloads"))
                                        .unwrap_or_else(|| PathBuf::from("."))
                                });
                                *destination = download_dir.join(filename);
                                true
                            }
                        }
                        tauri::webview::DownloadEvent::Finished { success, path, .. } => {
                            if success {
                                println!("[Appify] HTTP download finished: {:?}", path);
                            }
                            true
                        }
                        _ => true,
                    }
                })
                .build()?;

            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::*;
                use javascriptcore::ValueExt;
                use webkit2gtk::{PermissionRequestExt, SettingsExt, UserContentManagerExt, WebViewExt};

                let w_for_webview = window.clone();
                let css_for_ucm = effective_css_for_linux.clone();
                let _ = window.with_webview(move |platform_webview| {
                    let wv = platform_webview.inner();
                    if let Some(settings) = WebViewExt::settings(&wv) {
                        settings.set_enable_developer_extras(true);
                        settings.set_javascript_can_access_clipboard(true);
                        settings.set_enable_media_stream(true);
                        settings.set_enable_mediasource(true);
                        settings.set_enable_webrtc(true);
                        settings.set_enable_webaudio(true);
                        settings.set_enable_webgl(true);
                        settings.set_enable_encrypted_media(true);
                        settings.set_enable_media(true);
                        settings.set_enable_media_capabilities(true);
                        settings.set_media_playback_requires_user_gesture(false);
                        settings.set_media_playback_allows_inline(true);
                        settings.set_enable_fullscreen(true);
                        settings.set_enable_site_specific_quirks(true);
                        settings.set_enable_html5_database(true);
                        settings.set_enable_html5_local_storage(true);
                        settings.set_allow_file_access_from_file_urls(true);
                        settings.set_allow_universal_access_from_file_urls(true);
                    }
                    wv.connect_permission_request(|_wv, req| {
                        req.allow();
                        true
                    });

                    let devtools_toggle_instant = std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(10)));
                    let last_toggle_msg = devtools_toggle_instant.clone();
                    let last_toggle_key = devtools_toggle_instant.clone();
                    let active_blob_downloads: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, ActiveBlobDownload>>> =
                        std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

                    // Register custom CSS and script message handler
                    if let Some(ucm) = wv.user_content_manager() {
                        if !css_for_ucm.trim().is_empty() {
                            let uss = webkit2gtk::UserStyleSheet::new(
                                &css_for_ucm,
                                webkit2gtk::UserContentInjectedFrames::AllFrames,
                                webkit2gtk::UserStyleLevel::User,
                                &[],
                                &[],
                            );
                            ucm.add_style_sheet(&uss);
                        }
                        let _ = ucm.register_script_message_handler("appify");
                        let w_for_msg = w_for_webview.clone();
                        let active_dls = active_blob_downloads.clone();
                        let wv_for_msg = wv.clone();
                        ucm.connect_script_message_received(Some("appify"), move |_ucm, js_res| {
                            if let Some(val) = js_res.js_value() {
                                let msg = val.to_str();
                                match msg.as_str() {
                                    "toggle_devtools" => {
                                        let mut last = last_toggle_msg.lock().unwrap();
                                        if last.elapsed() >= std::time::Duration::from_millis(400) {
                                            *last = std::time::Instant::now();
                                            if w_for_msg.is_devtools_open() {
                                                w_for_msg.close_devtools();
                                            } else {
                                                w_for_msg.open_devtools();
                                            }
                                        }
                                    }
                                    "check_clipboard_paste" => {
                                        if let Some(png_bytes) = get_clipboard_image_png() {
                                            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                                            let js = format!("window.__appify_dispatch_paste_image('{}');", b64);
                                            let _ = w_for_msg.eval(&js);
                                        }
                                    }
                                    _ => {
                                        if msg.starts_with('{') {
                                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg) {
                                                let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                                match msg_type {
                                                    "download_request" => {
                                                        let transfer_id = parsed.get("transferId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                        let filename = parsed.get("filename").and_then(|v| v.as_str()).unwrap_or("download").to_string();
                                                        let total_size = parsed.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                                                        let parent_win = wv_for_msg.toplevel().and_then(|w| w.downcast::<gtk::Window>().ok());
                                                        if let Some(dest_path) = prompt_save_file_picker(parent_win.as_ref(), &filename) {
                                                            if let Some(parent_dir) = dest_path.parent() {
                                                                let _ = fs::create_dir_all(parent_dir);
                                                            }
                                                            match fs::File::create(&dest_path) {
                                                                Ok(file) => {
                                                                    let mut map = active_dls.lock().unwrap();
                                                                    map.insert(transfer_id.clone(), ActiveBlobDownload {
                                                                        destination: dest_path,
                                                                        file: Some(file),
                                                                        received_bytes: 0,
                                                                        total_bytes: total_size,
                                                                    });
                                                                    let js = format!("window.__appify_download_start_transfer('{}');", transfer_id);
                                                                    let _ = w_for_msg.eval(&js);
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("[Appify] Failed to create download target: {}", e);
                                                                    let js = format!("window.__appify_download_cancel('{}');", transfer_id);
                                                                    let _ = w_for_msg.eval(&js);
                                                                }
                                                            }
                                                        } else {
                                                            let js = format!("window.__appify_download_cancel('{}');", transfer_id);
                                                            let _ = w_for_msg.eval(&js);
                                                        }
                                                    }
                                                    "download_chunk" => {
                                                        let transfer_id = parsed.get("transferId").and_then(|v| v.as_str()).unwrap_or("");
                                                        let data_b64 = parsed.get("data").and_then(|v| v.as_str()).unwrap_or("");
                                                        let is_last = parsed.get("isLast").and_then(|v| v.as_bool()).unwrap_or(false);

                                                        if let Ok(bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data_b64) {
                                                            let mut map = active_dls.lock().unwrap();
                                                            if let Some(item) = map.get_mut(transfer_id) {
                                                                if let Some(file) = item.file.as_mut() {
                                                                    let _ = file.write_all(&bytes);
                                                                    item.received_bytes += bytes.len();
                                                                    if is_last {
                                                                        let _ = file.flush();
                                                                        let dest = item.destination.clone();
                                                                        map.remove(transfer_id);
                                                                        println!("[Appify] File downloaded: {}", dest.display());
                                                                        let _ = w_for_msg.request_user_attention(Some(tauri::UserAttentionType::Informational));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    "download_cancel" => {
                                                        let transfer_id = parsed.get("transferId").and_then(|v| v.as_str()).unwrap_or("");
                                                        let mut map = active_dls.lock().unwrap();
                                                        if let Some(item) = map.remove(transfer_id) {
                                                            if item.destination.exists() && item.received_bytes < item.total_bytes {
                                                                let _ = fs::remove_file(&item.destination);
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }

                    // Intercept keyboard shortcuts: F12, Ctrl+Shift+I/C/J, and Ctrl+V
                    let w_for_keys = w_for_webview.clone();
                    wv.connect_key_press_event(move |_wv, event_key| {
                        let keyval = event_key.keyval();
                        let state = event_key.state();
                        let is_ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
                        let is_shift = state.contains(gdk::ModifierType::SHIFT_MASK);

                        // DevTools shortcuts: F12, Ctrl+Shift+I, Ctrl+Shift+C, Ctrl+Shift+J
                        if keyval == gdk::keys::constants::F12
                            || (is_ctrl && is_shift && (
                                keyval == gdk::keys::constants::I || keyval == gdk::keys::constants::i
                                || keyval == gdk::keys::constants::C || keyval == gdk::keys::constants::c
                                || keyval == gdk::keys::constants::J || keyval == gdk::keys::constants::j
                            ))
                        {
                            let mut last = last_toggle_key.lock().unwrap();
                            if last.elapsed() >= std::time::Duration::from_millis(400) {
                                *last = std::time::Instant::now();
                                if w_for_keys.is_devtools_open() {
                                    w_for_keys.close_devtools();
                                } else {
                                    w_for_keys.open_devtools();
                                }
                            }
                            return glib::Propagation::Stop;
                        }

                        // Ctrl+V: Image paste preparation
                        if is_ctrl && !is_shift && (keyval == gdk::keys::constants::v || keyval == gdk::keys::constants::V) {
                            if let Some(png_bytes) = get_clipboard_image_png() {
                                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                                let js = format!("window.__appify_clipboard_image = '{}';", b64);
                                let _ = w_for_keys.eval(&js);
                            } else {
                                let _ = w_for_keys.eval("window.__appify_clipboard_image = null;");
                            }
                            return glib::Propagation::Proceed;
                        }

                        glib::Propagation::Proceed
                    });

                    let w_for_focus = w_for_webview.clone();
                    wv.connect_focus_in_event(move |_wv, _event| {
                        if let Some(png_bytes) = get_clipboard_image_png() {
                            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                            let js = format!("window.__appify_clipboard_image = '{}';", b64);
                            let _ = w_for_focus.eval(&js);
                        }
                        glib::Propagation::Proceed
                    });
                });
            }

            let _ = window.set_title(&win_title);

            if let Some(zoom_val) = meta.zoom {
                let _ = window.set_zoom(zoom_val);
            }

            let w_for_event = window.clone();
            let hide_on_close = meta.hide_on_close;
            window.on_window_event(move |event| {
                if hide_on_close {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w_for_event.hide();
                    }
                }
                if let tauri::WindowEvent::DragDrop(drag_event) = event {
                    if let tauri::DragDropEvent::Drop { paths, .. } = drag_event {
                        let mut files_json = Vec::new();
                        for p in paths {
                            if let Ok(bytes) = fs::read(p) {
                                let filename = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                                let ext = p.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                                let mime = match ext.as_str() {
                                    "png" => "image/png",
                                    "jpg" | "jpeg" => "image/jpeg",
                                    "webp" => "image/webp",
                                    "gif" => "image/gif",
                                    "bmp" => "image/bmp",
                                    _ => "application/octet-stream",
                                };
                                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
                                files_json.push(serde_json::json!({
                                    "name": filename,
                                    "type": mime,
                                    "data": b64,
                                }));
                            }
                        }
                        if !files_json.is_empty() {
                            if let Ok(payload) = serde_json::to_string(&files_json) {
                                let js = format!("window.__appify_handle_dropped_files({});", payload);
                                let _ = w_for_event.eval(&js);
                            }
                        }
                    }
                }
            });

            let icon_img = if let Some(ref icon_path_str) = meta.custom_icon {
                let p = Path::new(icon_path_str);
                if p.exists() {
                    fs::read(p).ok().and_then(|b| {
                        tauri::image::Image::from_bytes(&b).ok().or_else(|| {
                            convert_to_png(&b).and_then(|png| tauri::image::Image::from_bytes(&png).ok())
                        })
                    })
                } else {
                    None
                }
            } else {
                None
            }
            .or_else(|| app.default_window_icon().cloned())
            .or_else(|| tauri::image::Image::from_bytes(FALLBACK_ICON_BYTES).ok());

            if let Some(ref icon) = icon_img {
                let _ = window.set_icon(icon.clone());
            }

            if meta.tray {
                if let Some(tray_icon) = icon_img {
                    let show_hide_item = tauri::menu::MenuItem::with_id(app, "toggle", "Show / Hide", true, None::<&str>)?;
                    let reload_item = tauri::menu::MenuItem::with_id(app, "reload", "Reload", true, None::<&str>)?;
                    let devtools_item = tauri::menu::MenuItem::with_id(app, "devtools", "Toggle Developer Tools", true, None::<&str>)?;
                    let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                    let menu = tauri::menu::Menu::with_items(app, &[&show_hide_item, &reload_item, &devtools_item, &quit_item])?;

                    let _tray = tauri::tray::TrayIconBuilder::new()
                        .icon(tray_icon)
                        .tooltip(&win_title)
                        .menu(&menu)
                        .show_menu_on_left_click(false)
                        .on_menu_event(move |app_handle, event| {
                            match event.id.as_ref() {
                                "toggle" => {
                                    if let Some(w) = app_handle.get_webview_window("main") {
                                        if w.is_visible().unwrap_or(false) {
                                            let _ = w.hide();
                                        } else {
                                            let _ = w.show();
                                            let _ = w.unminimize();
                                            let _ = w.set_focus();
                                        }
                                    }
                                }
                                "reload" => {
                                    if let Some(w) = app_handle.get_webview_window("main") {
                                        let _ = w.reload();
                                    }
                                }
                                "devtools" => {
                                    if let Some(w) = app_handle.get_webview_window("main") {
                                        if w.is_devtools_open() {
                                            w.close_devtools();
                                        } else {
                                            w.open_devtools();
                                        }
                                    }
                                }
                                "quit" => {
                                    app_handle.exit(0);
                                }
                                _ => {}
                            }
                        })
                        .on_tray_icon_event(|tray, event| {
                            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                                let app = tray.app_handle();
                                if let Some(w) = app.get_webview_window("main") {
                                    if w.is_visible().unwrap_or(false) {
                                        let _ = w.hide();
                                    } else {
                                        let _ = w.show();
                                        let _ = w.unminimize();
                                        let _ = w.set_focus();
                                    }
                                }
                            }
                        })
                        .build(app)?;
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Appify instance");
}

fn get_persistent_bin_path() -> PathBuf {
    #[cfg(windows)]
    {
        let local_app_data = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join("AppData").join("Local"));
        local_app_data.join("Programs").join("appify").join("appify.exe")
    }
    #[cfg(not(windows))]
    {
        let home = dirs::home_dir().expect("Cannot resolve home directory");
        home.join(".local/bin/appify")
    }
}

fn copy_atomic(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    let pid = std::process::id();
    let tmp_dst = dst.with_extension(format!("tmp.{}", pid));
    fs::copy(src, &tmp_dst)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = fs::metadata(&tmp_dst).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&tmp_dst, perms);
        }
    }
    fs::rename(&tmp_dst, dst)?;
    Ok(())
}

fn is_dir_in_path(dir: &Path) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        std::env::split_paths(&path_var).any(|p| {
            if let (Ok(p1), Ok(p2)) = (fs::canonicalize(&p), fs::canonicalize(dir)) {
                p1 == p2
            } else {
                p == dir
            }
        })
    } else {
        false
    }
}

fn ensure_persistent_binary() -> PathBuf {
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return PathBuf::from("appify"),
    };

    let persistent = get_persistent_bin_path();

    let is_already_persistent = if let (Ok(curr_can), Ok(pers_can)) =
        (fs::canonicalize(&current_exe), fs::canonicalize(&persistent))
    {
        curr_can == pers_can
    } else {
        current_exe == persistent
    };

    let is_system_bin = current_exe.starts_with("/usr/bin")
        || current_exe.starts_with("/usr/local/bin")
        || current_exe.starts_with("/opt");

    if is_already_persistent || is_system_bin {
        return current_exe;
    }

    println!(
        "[*] Relocating binary to '{}' so desktop launchers remain permanent...",
        persistent.display()
    );
    if let Err(e) = copy_atomic(&current_exe, &persistent) {
        eprintln!(
            "[!] Warning: Could not install binary to persistent location ({}). Using current executable.",
            e
        );
        return current_exe;
    }

    if let Some(parent) = persistent.parent() {
        if !is_dir_in_path(parent) {
            println!("[!] Note: '{}' is not currently in your PATH.", parent.display());
            #[cfg(unix)]
            println!("    To run 'appify' from any terminal, add it to your shell profile:\n    echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.bashrc");
            #[cfg(windows)]
            println!("    To run 'appify' from any terminal, add '{}' to your User PATH.", parent.display());
        }
    }

    persistent
}

fn execute_self_install() -> PathBuf {
    let persistent = ensure_persistent_binary();
    println!("[+] Appify is installed at: {}", persistent.display());

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let apps_dir = home.join(".local/share/applications");
    let icons_dir = home.join(".local/share/icons");
    fs::create_dir_all(&apps_dir).ok();
    fs::create_dir_all(&icons_dir).ok();

    let manager_icon = icons_dir.join("appify.png");
    let _ = fs::write(&manager_icon, FALLBACK_ICON_BYTES);

    let desktop_content = format!(
        "[Desktop Entry]\n\
        Version=1.0\n\
        Type=Application\n\
        Name=Appify Manager\n\
        Comment=Turn web applications into isolated native desktop apps\n\
        Exec={bin} ui\n\
        Icon={icon}\n\
        Terminal=false\n\
        Categories=Utility;Network;\n\
        StartupWMClass=appify\n",
        bin = persistent.display(),
        icon = manager_icon.display()
    );
    let _ = fs::write(apps_dir.join("appify.desktop"), desktop_content);
    println!("[+] Appify Manager launcher created at: {}", apps_dir.join("appify.desktop").display());

    persistent
}

fn install_autostart(
    meta: &AppMetadata,
    bin_path: &Path,
    icon_path: &Path,
    is_hidden: bool,
) -> Result<PathBuf, String> {
    let mut flags = Vec::new();
    if meta.hide_on_close {
        flags.push("--hide-on-close".to_string());
    }
    if meta.single_instance {
        flags.push("--single-instance".to_string());
    }
    if meta.tray || is_hidden {
        flags.push("--tray".to_string());
    }
    if is_hidden {
        flags.push("--start-hidden".to_string());
    }
    if meta.maximize {
        flags.push("--maximize".to_string());
    }
    if let Some(z) = meta.zoom {
        flags.push(format!("--zoom {}", z));
    }
    if let Some(ref ua) = meta.user_agent {
        flags.push(format!("--user-agent \"{}\"", ua));
    }
    if let Some(w) = meta.width {
        flags.push(format!("--width {}", w));
    }
    if let Some(h) = meta.height {
        flags.push(format!("--height {}", h));
    }
    for d in &meta.custom_allowed_domains {
        flags.push(format!("--allow-domain \"{}\"", d));
    }

    let extra_args = if !flags.is_empty() {
        format!(" {}", flags.join(" "))
    } else {
        String::new()
    };

    #[cfg(target_os = "linux")]
    {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .ok_or("Cannot find config directory")?;
        let autostart_dir = config_dir.join("autostart");
        fs::create_dir_all(&autostart_dir).map_err(|e| e.to_string())?;
        let target_file = autostart_dir.join(format!("appify-{}.desktop", &meta.hash));

        let content = format!(
            "[Desktop Entry]\n\
            Version=1.0\n\
            Type=Application\n\
            Name={name}\n\
            Exec={bin} run \"{url}\" \"{name}\" --wm-class \"{wm_class}\" --icon \"{icon}\"{extra_args}\n\
            Icon={icon}\n\
            Terminal=false\n\
            Categories=Network;\n\
            StartupWMClass={wm_class}\n\
            X-GNOME-Autostart-enabled=true\n\
            X-Appify-URL={url}\n\
            X-Appify-Hash={hash}\n\
            X-Appify-Autostart=true\n",
            name = meta.name,
            bin = bin_path.display(),
            url = meta.url,
            wm_class = meta.wm_class,
            icon = icon_path.display(),
            extra_args = extra_args,
            hash = meta.hash
        );
        fs::write(&target_file, content).map_err(|e| e.to_string())?;
        Ok(target_file)
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let agents_dir = home.join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;
        let target_file = agents_dir.join(format!("com.appify.{}.plist", &meta.hash));

        let mut xml_args = format!(
            "        <string>{}</string>\n        <string>run</string>\n        <string>{}</string>\n        <string>{}</string>\n        <string>--wm-class</string>\n        <string>{}</string>\n        <string>--icon</string>\n        <string>{}</string>\n",
            bin_path.display(),
            meta.url,
            meta.name,
            meta.wm_class,
            icon_path.display()
        );
        for flag in &flags {
            if let Some((f, val)) = flag.split_once(' ') {
                let clean_val = val.trim_matches('"');
                xml_args.push_str(&format!(
                    "        <string>{}</string>\n        <string>{}</string>\n",
                    f, clean_val
                ));
            } else {
                xml_args.push_str(&format!("        <string>{}</string>\n", flag));
            }
        }

        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
            <plist version=\"1.0\">\n\
            <dict>\n\
                <key>Label</key>\n\
                <string>com.appify.{hash}</string>\n\
                <key>ProgramArguments</key>\n\
                <array>\n\
            {args}\
                </array>\n\
                <key>RunAtLoad</key>\n\
                <true/>\n\
                <key>ProcessType</key>\n\
                <string>Interactive</string>\n\
            </dict>\n\
            </plist>\n",
            hash = meta.hash,
            args = xml_args
        );
        fs::write(&target_file, content).map_err(|e| e.to_string())?;
        Ok(target_file)
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = dirs::config_dir().ok_or("Cannot find APPDATA directory")?;
        let startup_dir = appdata.join("Microsoft/Windows/Start Menu/Programs/Startup");
        fs::create_dir_all(&startup_dir).map_err(|e| e.to_string())?;
        let target_file = startup_dir.join(format!("appify-{}.vbs", &meta.hash));

        let full_cmd = format!(
            "\"{}\" run \"{}\" \"{}\" --wm-class \"{}\" --icon \"{}\"{}",
            bin_path.display(),
            meta.url,
            meta.name,
            meta.wm_class,
            icon_path.display(),
            extra_args
        );
        let escaped_cmd = full_cmd.replace('"', "\"\"");
        let content = format!(
            "Set WshShell = CreateObject(\"WScript.Shell\")\r\n\
            WshShell.Run \"{escaped}\", 0, False\r\n",
            escaped = escaped_cmd
        );
        fs::write(&target_file, content).map_err(|e| e.to_string())?;
        Ok(target_file)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Autostart is not supported on this operating system".to_string())
    }
}

fn remove_autostart(hash: &str) -> bool {
    let mut removed = false;
    #[cfg(target_os = "linux")]
    {
        if let Some(config_dir) = dirs::config_dir().or_else(|| dirs::home_dir().map(|h| h.join(".config"))) {
            let p = config_dir.join("autostart").join(format!("appify-{}.desktop", hash));
            if p.exists() {
                let _ = fs::remove_file(&p);
                removed = true;
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let p = home.join("Library/LaunchAgents").join(format!("com.appify.{}.plist", hash));
            if p.exists() {
                let _ = std::process::Command::new("launchctl")
                    .arg("unload")
                    .arg(&p)
                    .output();
                let _ = fs::remove_file(&p);
                removed = true;
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = dirs::config_dir() {
            let startup_dir = appdata.join("Microsoft/Windows/Start Menu/Programs/Startup");
            let vbs = startup_dir.join(format!("appify-{}.vbs", hash));
            let bat = startup_dir.join(format!("appify-{}.bat", hash));
            if vbs.exists() {
                let _ = fs::remove_file(&vbs);
                removed = true;
            }
            if bat.exists() {
                let _ = fs::remove_file(&bat);
                removed = true;
            }
        }
    }
    removed
}

fn get_autostart_status(hash: &str) -> &'static str {
    #[cfg(target_os = "linux")]
    {
        if let Some(config_dir) = dirs::config_dir().or_else(|| dirs::home_dir().map(|h| h.join(".config"))) {
            let p = config_dir.join("autostart").join(format!("appify-{}.desktop", hash));
            if p.exists() {
                if let Ok(c) = fs::read_to_string(&p) {
                    if c.contains("--start-hidden") {
                        return "Enabled (Tray)";
                    }
                }
                return "Enabled";
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let p = home.join("Library/LaunchAgents").join(format!("com.appify.{}.plist", hash));
            if p.exists() {
                if let Ok(c) = fs::read_to_string(&p) {
                    if c.contains("<string>--start-hidden</string>") {
                        return "Enabled (Tray)";
                    }
                }
                return "Enabled";
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = dirs::config_dir() {
            let vbs = appdata.join("Microsoft/Windows/Start Menu/Programs/Startup").join(format!("appify-{}.vbs", hash));
            if vbs.exists() {
                if let Ok(c) = fs::read_to_string(&vbs) {
                    if c.contains("--start-hidden") {
                        return "Enabled (Tray)";
                    }
                }
                return "Enabled";
            }
        }
    }
    "Disabled"
}

fn get_profile_dir(hash: &str) -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("appify")
        .join(hash)
}

fn migrate_legacy_profiles() {
    let base_appify = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("appify");
    let legacy_profiles_dir = base_appify.join("profiles");
    if legacy_profiles_dir.exists() && legacy_profiles_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&legacy_profiles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let hash_str = entry.file_name().to_string_lossy().to_string();
                    let target_dir = base_appify.join(&hash_str);
                    if !target_dir.exists() {
                        let _ = fs::rename(&path, &target_dir);
                    }
                }
            }
        }
        let _ = fs::remove_dir_all(&legacy_profiles_dir);
    }

    // Ensure icon.png and preset default scripts exist in each app directory
    if let Ok(entries) = fs::read_dir(&base_appify) {
        let home_icons = dirs::home_dir().map(|h| h.join(".local/share/icons"));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let hash_str = entry.file_name().to_string_lossy().to_string();
                let icon_dest = path.join("icon.png");
                if !icon_dest.exists() {
                    if let Some(ref h_icons) = home_icons {
                        let sys_icon = h_icons.join(format!("appify-{}.png", &hash_str));
                        if sys_icon.exists() {
                            let _ = fs::copy(sys_icon, &icon_dest);
                        }
                    }
                }

                // Check config.json to detect presets and inject default scripts if not present
                let cfg_path = path.join("config.json");
                if let Ok(content) = fs::read_to_string(&cfg_path) {
                    if let Ok(mut meta) = serde_json::from_str::<AppMetadata>(&content) {
                        let presets = get_curated_presets();
                        if let Some(p) = presets.iter().find(|p| p.url == meta.url || p.id == meta.base_domain) {
                            if !icon_dest.exists() {
                                if let Some(bytes) = p.bundled_icon {
                                    let _ = fs::write(&icon_dest, bytes);
                                }
                            }
                            let js_dest = path.join("userscript.js");
                            if !js_dest.exists() && p.default_inject_js.is_some() {
                                if let Some(js) = p.default_inject_js {
                                    let _ = fs::write(&js_dest, js);
                                    meta.inject_js = Some(js.to_string());
                                }
                            }
                            let css_dest = path.join("userstyle.css");
                            if !css_dest.exists() && p.default_inject_css.is_some() {
                                if let Some(css) = p.default_inject_css {
                                    let _ = fs::write(&css_dest, css);
                                    meta.inject_css = Some(css.to_string());
                                }
                            }
                            if meta.app_plugins.is_empty() && (meta.url.contains("whatsapp") || meta.base_domain.contains("whatsapp")) {
                                meta.app_plugins.push(AppExtensionRef {
                                    id: "builtin-wa-promo-cleaner".to_string(),
                                    name: "WhatsApp Promo Cleaner".to_string(),
                                    category: "plugin".to_string(),
                                    enabled: true,
                                    custom_content: None,
                                });
                            }
                            if meta.app_themes.is_empty() && (meta.url.contains("whatsapp") || meta.base_domain.contains("whatsapp")) {
                                meta.app_themes.push(AppExtensionRef {
                                    id: "builtin-wa-promo-css".to_string(),
                                    name: "WhatsApp Clean Layout".to_string(),
                                    category: "theme".to_string(),
                                    enabled: true,
                                    custom_content: None,
                                });
                            }
                            if let Ok(new_json) = serde_json::to_string_pretty(&meta) {
                                let _ = fs::write(&cfg_path, new_json);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_builtin_extensions() -> ExtensionRegistry {
    ExtensionRegistry {
        plugins: vec![
            ExtensionItem {
                id: "builtin-wa-promo-cleaner".to_string(),
                name: "WhatsApp Promo Cleaner".to_string(),
                description: "Removes 'Get WhatsApp for Mac / Windows' buttons and promo rails".to_string(),
                category: "plugin".to_string(),
                content: r#"
(function() {
    function cleanWhatsAppDownloadPromos() {
        var selectors = [
            'a[href*="whatsapp.com/download"]',
            'a[href*="microsoft.com/store"][href*="whatsapp"]',
            'button[aria-label*="Get WhatsApp" i]',
            'button[aria-label*="Get the app" i]',
            'div[aria-label*="Get WhatsApp" i]',
            'span[aria-label*="Get WhatsApp" i]',
            'div[data-testid*="intro-banner"]',
            'div[data-testid*="native-desktop-banner"]',
            'div[data-testid*="desktop-app-banner"]',
            'div[data-testid*="get-desktop-app"]'
        ];
        var els = document.querySelectorAll(selectors.join(','));
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            var btn = el.closest('div[role="button"]') || el.closest('button') || el;
            btn.style.setProperty('display', 'none', 'important');
        }

        try {
            var walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
            var node;
            var toHide = [];
            while (node = walker.nextNode()) {
                var text = node.textContent.trim();
                if (/Get WhatsApp for (Mac|Windows)/i.test(text) || /^Get WhatsApp$/i.test(text)) {
                    var container = node.parentElement;
                    for (var k = 0; k < 5 && container && container !== document.body; k++) {
                        if (container.getAttribute('role') === 'button' || container.tagName === 'BUTTON' || container.tagName === 'A') {
                            break;
                        }
                        container = container.parentElement;
                    }
                    if (container) toHide.push(container);
                }
            }
            for (var j = 0; j < toHide.length; j++) {
                toHide[j].style.setProperty('display', 'none', 'important');
            }
        } catch (e) {}
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', cleanWhatsAppDownloadPromos);
    } else {
        cleanWhatsAppDownloadPromos();
    }

    var observer = new MutationObserver(cleanWhatsAppDownloadPromos);
    observer.observe(document.documentElement, { childList: true, subtree: true });
})();
"#.trim().to_string(),
                author: "Appify Core".to_string(),
                version: "1.0.0".to_string(),
                is_builtin: true,
            },
            ExtensionItem {
                id: "builtin-dark-mode-helper".to_string(),
                name: "Emulate Dark Theme".to_string(),
                description: "Emulates prefers-color-scheme: dark media query for supported web apps".to_string(),
                category: "plugin".to_string(),
                content: r#"
(function() {
    try {
        var originalMatchMedia = window.matchMedia;
        window.matchMedia = function(query) {
            if (query && query.indexOf('prefers-color-scheme: dark') !== -1) {
                return {
                    matches: true,
                    media: query,
                    onchange: null,
                    addListener: function() {},
                    removeListener: function() {},
                    addEventListener: function() {},
                    removeEventListener: function() {},
                    dispatchEvent: function() { return false; }
                };
            }
            return originalMatchMedia.apply(window, arguments);
        };
    } catch(e) {}
})();
"#.trim().to_string(),
                author: "Appify Core".to_string(),
                version: "1.0.0".to_string(),
                is_builtin: true,
            },
            ExtensionItem {
                id: "builtin-console-logger".to_string(),
                name: "Console Navigation Logger".to_string(),
                description: "Logs page loads, titles, and navigation changes in Developer Tools console".to_string(),
                category: "plugin".to_string(),
                content: r#"
(function() {
    console.log("%c[Appify]%c App initialized: " + document.title + " (" + window.location.href + ")", "color: #3b82f6; font-weight: bold;", "color: inherit;");
})();
"#.trim().to_string(),
                author: "Appify Core".to_string(),
                version: "1.0.0".to_string(),
                is_builtin: true,
            },
        ],
        themes: vec![
            ExtensionItem {
                id: "builtin-wa-promo-css".to_string(),
                name: "WhatsApp Clean Layout".to_string(),
                description: "Hides 'Get WhatsApp for Mac / Windows' promos and download banners via CSS".to_string(),
                category: "theme".to_string(),
                content: r#"
/* Hide 'Get WhatsApp for Mac / Windows' desktop app promo banners */
a[href*="whatsapp.com/download"],
a[href*="microsoft.com/store"][href*="whatsapp"],
button[aria-label*="Get WhatsApp" i],
button[aria-label*="Get the app" i],
button[title*="Get WhatsApp" i],
div[aria-label*="Get WhatsApp" i],
span[aria-label*="Get WhatsApp" i],
div[data-testid*="intro-banner"],
div[data-testid*="native-desktop-banner"],
div[data-testid*="desktop-app-banner"],
div[data-testid*="get-desktop-app"] {
    display: none !important;
}
"#.trim().to_string(),
                author: "Appify Core".to_string(),
                version: "1.0.0".to_string(),
                is_builtin: true,
            },
            ExtensionItem {
                id: "builtin-hide-scrollbars".to_string(),
                name: "Minimalist Hidden Scrollbars".to_string(),
                description: "Hides all WebKit scrollbars while preserving mouse and trackpad scrolling".to_string(),
                category: "theme".to_string(),
                content: r#"
/* Hide WebKit scrollbars */
::-webkit-scrollbar {
    width: 0px !important;
    height: 0px !important;
    display: none !important;
}
* {
    scrollbar-width: none !important;
}
"#.trim().to_string(),
                author: "Appify Core".to_string(),
                version: "1.0.0".to_string(),
                is_builtin: true,
            },
            ExtensionItem {
                id: "builtin-modern-dark".to_string(),
                name: "Subtle Dark Enhancer".to_string(),
                description: "Reduces pure-white backgrounds and smoothes font rendering".to_string(),
                category: "theme".to_string(),
                content: r#"
/* Smooth anti-aliasing */
body, html {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}
"#.trim().to_string(),
                author: "Appify Core".to_string(),
                version: "1.0.0".to_string(),
                is_builtin: true,
            },
        ],
    }
}

fn get_registry_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("appify")
        .join("registry.json")
}

fn load_registry() -> ExtensionRegistry {
    let builtins = get_builtin_extensions();
    let reg_path = get_registry_path();
    if !reg_path.exists() {
        let _ = save_registry(&builtins);
        return builtins;
    }

    let mut current: ExtensionRegistry = match fs::read_to_string(&reg_path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| builtins.clone()),
        Err(_) => builtins.clone(),
    };

    for b_plug in &builtins.plugins {
        if let Some(existing) = current.plugins.iter_mut().find(|p| p.id == b_plug.id) {
            existing.is_builtin = true;
            existing.content = b_plug.content.clone();
            existing.name = b_plug.name.clone();
        } else {
            current.plugins.push(b_plug.clone());
        }
    }

    for b_thm in &builtins.themes {
        if let Some(existing) = current.themes.iter_mut().find(|t| t.id == b_thm.id) {
            existing.is_builtin = true;
            existing.content = b_thm.content.clone();
            existing.name = b_thm.name.clone();
        } else {
            current.themes.push(b_thm.clone());
        }
    }

    current
}

fn save_registry(registry: &ExtensionRegistry) -> Result<(), String> {
    let reg_path = get_registry_path();
    if let Some(parent) = reg_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(reg_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn compile_app_extensions(meta: &mut AppMetadata) {
    let registry = load_registry();
    let mut compiled_js = Vec::new();
    let mut compiled_css = Vec::new();

    for ext in &meta.app_plugins {
        if ext.enabled {
            if let Some(ref custom) = ext.custom_content {
                if !custom.trim().is_empty() {
                    compiled_js.push(custom.clone());
                    continue;
                }
            }
            if let Some(reg_item) = registry.plugins.iter().find(|p| p.id == ext.id) {
                if !reg_item.content.trim().is_empty() {
                    compiled_js.push(reg_item.content.clone());
                }
            }
        }
    }

    for ext in &meta.app_themes {
        if ext.enabled {
            if let Some(ref custom) = ext.custom_content {
                if !custom.trim().is_empty() {
                    compiled_css.push(custom.clone());
                    continue;
                }
            }
            if let Some(reg_item) = registry.themes.iter().find(|t| t.id == ext.id) {
                if !reg_item.content.trim().is_empty() {
                    compiled_css.push(reg_item.content.clone());
                }
            }
        }
    }

    if !compiled_js.is_empty() {
        meta.inject_js = Some(compiled_js.join("\n\n"));
    } else if !meta.app_plugins.is_empty() {
        meta.inject_js = None;
    }

    if !compiled_css.is_empty() {
        meta.inject_css = Some(compiled_css.join("\n\n"));
    } else if !meta.app_themes.is_empty() {
        meta.inject_css = None;
    }
}

const MARKETPLACE_CDN_URL: &str = "https://cdn.appify.aerovex.net/v1/marketplace.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceItemDto {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub category: String, // "plugin" or "theme"
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub author_url: Option<String>,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub target_domains: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub size_bytes: usize,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub preview_code: Option<String>,
    #[serde(default)]
    pub code_content: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketplaceCatalogDto {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub cdn_base: String,
    #[serde(default)]
    pub generated_at: String,
    #[serde(default)]
    pub total_items: usize,
    #[serde(default)]
    pub plugins_count: usize,
    #[serde(default)]
    pub themes_count: usize,
    #[serde(default)]
    pub items: Vec<MarketplaceItemDto>,
}

fn get_marketplace_cache_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("appify")
        .join("marketplace_cache.json")
}

fn get_starter_marketplace_catalog() -> MarketplaceCatalogDto {
    MarketplaceCatalogDto {
        version: "1.0.0".to_string(),
        cdn_base: "https://cdn.appify.aerovex.net".to_string(),
        generated_at: "2026-09-22T20:00:00Z".to_string(),
        total_items: 4,
        plugins_count: 2,
        themes_count: 2,
        items: vec![
            MarketplaceItemDto {
                id: "github-wide-diffs".to_string(),
                name: "GitHub Wide Diffs".to_string(),
                description: "Expands GitHub diffs, pull request code viewers, and table containers across the full screen width.".to_string(),
                category: "plugin".to_string(),
                author: "Aerovex".to_string(),
                author_url: Some("https://github.com/aerovexsim".to_string()),
                version: "1.0.0".to_string(),
                target_domains: vec!["github.com".to_string()],
                tags: vec!["github".to_string(), "developer".to_string(), "layout".to_string(), "productive".to_string()],
                icon: None,
                sha256: "12560807ebc3ff4beee0a6aee7039537f7c46f1b3652c7ef696d5e1ba0f742fa".to_string(),
                size_bytes: 486,
                download_url: "https://cdn.appify.aerovex.net/v1/plugins/github-wide-diffs.js".to_string(),
                preview_code: Some(r#"// GitHub Wide Diffs Plugin for Appify
(function () {
  const applyWideLayout = () => {
    const containers = document.querySelectorAll(
      ".container-xl, .container-lg, .diff-view, .pull-request-tab-content"
    );
    containers.forEach((el) => {
      el.style.setProperty("max-width", "98%", "important");
      el.style.setProperty("width", "98%", "important");
    });
  };

  window.addEventListener("load", applyWideLayout);
  const observer = new MutationObserver(applyWideLayout);
  observer.observe(document.body, { childList: true, subtree: true });
  applyWideLayout();
})();"#.to_string()),
                code_content: Some(r#"// GitHub Wide Diffs Plugin for Appify
(function () {
  const applyWideLayout = () => {
    const containers = document.querySelectorAll(
      ".container-xl, .container-lg, .diff-view, .pull-request-tab-content"
    );
    containers.forEach((el) => {
      el.style.setProperty("max-width", "98%", "important");
      el.style.setProperty("width", "98%", "important");
    });
  };

  window.addEventListener("load", applyWideLayout);
  const observer = new MutationObserver(applyWideLayout);
  observer.observe(document.body, { childList: true, subtree: true });
  applyWideLayout();
})();"#.to_string()),
                created_at: Some("2026-09-22T20:00:00Z".to_string()),
                updated_at: Some("2026-09-22T20:00:00Z".to_string()),
            },
            MarketplaceItemDto {
                id: "youtube-ambient-control".to_string(),
                name: "YouTube Ambient Enhancer".to_string(),
                description: "Optimizes playback performance by dampening high-GPU cinematic ambient glows and improving video framing.".to_string(),
                category: "plugin".to_string(),
                author: "Aerovex".to_string(),
                author_url: Some("https://github.com/aerovexsim".to_string()),
                version: "1.0.0".to_string(),
                target_domains: vec!["youtube.com".to_string(), "music.youtube.com".to_string()],
                tags: vec!["youtube".to_string(), "video".to_string(), "performance".to_string(), "media".to_string()],
                icon: None,
                sha256: "36720f4ea24dfc0ffae6832dbca7452d3a1f114c99c8bf49f7b6070624bc5594".to_string(),
                size_bytes: 310,
                download_url: "https://cdn.appify.aerovex.net/v1/plugins/youtube-ambient-control.js".to_string(),
                preview_code: Some(r#"// YouTube Ambient Enhancer Plugin for Appify
(function () {
  const optimizePlayer = () => {
    const ambient = document.getElementById("cinematic-container");
    if (ambient) {
      ambient.style.setProperty("display", "none", "important");
    }
  };

  window.addEventListener("load", optimizePlayer);
  setInterval(optimizePlayer, 2000);
})();"#.to_string()),
                code_content: Some(r#"// YouTube Ambient Enhancer Plugin for Appify
(function () {
  const optimizePlayer = () => {
    const ambient = document.getElementById("cinematic-container");
    if (ambient) {
      ambient.style.setProperty("display", "none", "important");
    }
  };

  window.addEventListener("load", optimizePlayer);
  setInterval(optimizePlayer, 2000);
})();"#.to_string()),
                created_at: Some("2026-09-22T20:00:00Z".to_string()),
                updated_at: Some("2026-09-22T20:00:00Z".to_string()),
            },
            MarketplaceItemDto {
                id: "oled-true-black".to_string(),
                name: "OLED True Black".to_string(),
                description: "Transforms dark-gray web app backgrounds into pure #000000 black to maximize contrast and battery life on OLED screens.".to_string(),
                category: "theme".to_string(),
                author: "Aerovex".to_string(),
                author_url: Some("https://github.com/aerovexsim".to_string()),
                version: "1.0.0".to_string(),
                target_domains: vec!["*".to_string()],
                tags: vec!["oled".to_string(), "black".to_string(), "battery".to_string(), "dark".to_string(), "universal".to_string()],
                icon: None,
                sha256: "df85664eb27fc18be968748d56b0254cb6b03ebdfa1f280e5e04b4cfbf3f9ae2".to_string(),
                size_bytes: 204,
                download_url: "https://cdn.appify.aerovex.net/v1/themes/oled-true-black.css".to_string(),
                preview_code: Some(r#"/* OLED True Black Theme for Appify */
html, body {
  background-color: #000000 !important;
  color: #f1f5f9 !important;
}

[class*="dark"], [data-theme="dark"], body.dark {
  background-color: #000000 !important;
}"#.to_string()),
                code_content: Some(r#"/* OLED True Black Theme for Appify */
html, body {
  background-color: #000000 !important;
  color: #f1f5f9 !important;
}

[class*="dark"], [data-theme="dark"], body.dark {
  background-color: #000000 !important;
}"#.to_string()),
                created_at: Some("2026-09-22T20:00:00Z".to_string()),
                updated_at: Some("2026-09-22T20:00:00Z".to_string()),
            },
            MarketplaceItemDto {
                id: "nord-frost".to_string(),
                name: "Nord Frost Theme".to_string(),
                description: "Applies an arctic, north-bluish palette with smooth contrast and clean cool-gray accents.".to_string(),
                category: "theme".to_string(),
                author: "Aerovex".to_string(),
                author_url: Some("https://github.com/aerovexsim".to_string()),
                version: "1.0.0".to_string(),
                target_domains: vec!["*".to_string()],
                tags: vec!["nord".to_string(), "frost".to_string(), "palette".to_string(), "aesthetic".to_string(), "universal".to_string()],
                icon: None,
                sha256: "e751897d25e865fc2058a7413d70e4c5b364fe06d3fc291d9b30c1be7d28c8a1".to_string(),
                size_bytes: 257,
                download_url: "https://cdn.appify.aerovex.net/v1/themes/nord-frost.css".to_string(),
                preview_code: Some(r#"/* Nord Frost Arctic Theme for Appify */
:root {
  --nord0: #2e3440;
  --nord1: #3b4252;
  --nord2: #434c5e;
  --nord3: #4c566a;
  --nord4: #d8dee9;
  --nord5: #e5e9f0;
  --nord6: #eceff4;
  --nord7: #8fbcbb;
  --nord8: #88c0d0;
  --nord9: #81a1c1;
  --nord10: #5e81ac;
}

body {
  background-color: var(--nord0) !important;
  color: var(--nord6) !important;
}"#.to_string()),
                code_content: Some(r#"/* Nord Frost Arctic Theme for Appify */
:root {
  --nord0: #2e3440;
  --nord1: #3b4252;
  --nord2: #434c5e;
  --nord3: #4c566a;
  --nord4: #d8dee9;
  --nord5: #e5e9f0;
  --nord6: #eceff4;
  --nord7: #8fbcbb;
  --nord8: #88c0d0;
  --nord9: #81a1c1;
  --nord10: #5e81ac;
}

body {
  background-color: var(--nord0) !important;
  color: var(--nord6) !important;
}"#.to_string()),
                created_at: Some("2026-09-22T20:00:00Z".to_string()),
                updated_at: Some("2026-09-22T20:00:00Z".to_string()),
            },
        ],
    }
}

fn fetch_marketplace_catalog() -> Result<MarketplaceCatalogDto, String> {
    let resp = ureq::get(MARKETPLACE_CDN_URL)
        .timeout(std::time::Duration::from_millis(3500))
        .call();

    if let Ok(response) = resp {
        if let Ok(body) = response.into_string() {
            if let Ok(catalog) = serde_json::from_str::<MarketplaceCatalogDto>(&body) {
                let cache_path = get_marketplace_cache_path();
                if let Some(parent) = cache_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&cache_path, &body);
                return Ok(catalog);
            }
        }
    }

    let cache_path = get_marketplace_cache_path();
    if cache_path.exists() {
        if let Ok(s) = fs::read_to_string(&cache_path) {
            if let Ok(catalog) = serde_json::from_str::<MarketplaceCatalogDto>(&s) {
                return Ok(catalog);
            }
        }
    }

    Ok(get_starter_marketplace_catalog())
}

fn install_marketplace_item(id: &str) -> Result<ExtensionItem, String> {
    let catalog = fetch_marketplace_catalog()?;
    let item = catalog.items.into_iter().find(|i| i.id == id)
        .ok_or_else(|| format!("Marketplace extension '{}' not found", id))?;

    let code_content = if let Some(code) = item.code_content {
        code
    } else if !item.download_url.is_empty() {
        ureq::get(&item.download_url)
            .timeout(std::time::Duration::from_millis(5000))
            .call()
            .map_err(|e| format!("Failed to download extension code: {}", e))?
            .into_string()
            .map_err(|e| format!("Failed to read extension body: {}", e))?
    } else {
        return Err("No code content available for this extension".to_string());
    };

    let mut registry = load_registry();
    let ext_item = ExtensionItem {
        id: item.id.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        category: item.category.clone(),
        content: code_content,
        author: item.author.clone(),
        version: item.version.clone(),
        is_builtin: false,
    };

    if item.category == "plugin" {
        if let Some(pos) = registry.plugins.iter().position(|p| p.id == ext_item.id) {
            registry.plugins[pos] = ext_item.clone();
        } else {
            registry.plugins.push(ext_item.clone());
        }
    } else {
        if let Some(pos) = registry.themes.iter().position(|t| t.id == ext_item.id) {
            registry.themes[pos] = ext_item.clone();
        } else {
            registry.themes.push(ext_item.clone());
        }
    }

    save_registry(&registry)?;
    Ok(ext_item)
}

fn get_config_path(hash: &str) -> PathBuf {
    get_profile_dir(hash).join("config.json")
}

fn save_app_config(meta: &AppMetadata) {
    let mut meta = meta.clone();
    compile_app_extensions(&mut meta);

    let p_dir = get_profile_dir(&meta.hash);
    let _ = fs::create_dir_all(&p_dir);
    if let Ok(json) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(get_config_path(&meta.hash), json);
    }
    if let Some(ref js) = meta.inject_js {
        if !js.trim().is_empty() {
            let _ = fs::write(p_dir.join("userscript.js"), js);
        } else {
            let _ = fs::remove_file(p_dir.join("userscript.js"));
        }
    } else {
        let _ = fs::remove_file(p_dir.join("userscript.js"));
    }
    if let Some(ref css) = meta.inject_css {
        if !css.trim().is_empty() {
            let _ = fs::write(p_dir.join("userstyle.css"), css);
        } else {
            let _ = fs::remove_file(p_dir.join("userstyle.css"));
        }
    } else {
        let _ = fs::remove_file(p_dir.join("userstyle.css"));
    }
}

fn load_app_config(hash: &str) -> Option<AppMetadata> {
    let p_dir = get_profile_dir(hash);
    let config_file = get_config_path(hash);
    if config_file.exists() {
        if let Ok(content) = fs::read_to_string(&config_file) {
            if let Ok(mut cfg) = serde_json::from_str::<AppMetadata>(&content) {
                let js_file = p_dir.join("userscript.js");
                if js_file.exists() {
                    cfg.inject_js = fs::read_to_string(&js_file).ok();
                }
                let css_file = p_dir.join("userstyle.css");
                if css_file.exists() {
                    cfg.inject_css = fs::read_to_string(&css_file).ok();
                }
                return Some(cfg);
            }
        }
    }
    migrate_config_from_desktop(hash)
}

fn migrate_config_from_desktop(hash: &str) -> Option<AppMetadata> {
    let home = dirs::home_dir()?;
    let desktop_path = home.join(format!(".local/share/applications/appify-{}.desktop", hash));
    if !desktop_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&desktop_path).ok()?;
    let mut name = String::new();
    let mut wm_class = String::new();
    let mut url = String::new();
    let mut exec = String::new();

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("Name=") {
            name = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("StartupWMClass=") {
            wm_class = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("X-Appify-URL=") {
            url = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Exec=") {
            exec = val.trim().to_string();
        }
    }

    if url.is_empty() {
        return None;
    }

    let hide_on_close = exec.contains("--hide-on-close");
    let single_instance = exec.contains("--single-instance");
    let tray = exec.contains("--tray");
    let start_hidden = exec.contains("--start-hidden");
    let maximize = exec.contains("--maximize");

    let mut zoom = None;
    if let Some(pos) = exec.find("--zoom ") {
        let rest = &exec[pos + 7..];
        let val_str = rest.split_whitespace().next().unwrap_or("");
        zoom = val_str.parse::<f64>().ok();
    }

    let autostart_status = get_autostart_status(hash);
    let autostart = autostart_status.starts_with("Enabled");
    let autostart_hidden = autostart_status.contains("Hidden");

    let p_dir = get_profile_dir(hash);
    let inject_js = fs::read_to_string(p_dir.join("userscript.js")).ok();
    let inject_css = fs::read_to_string(p_dir.join("userstyle.css")).ok();

    let meta = resolve_metadata(
        &url,
        RunOptions {
            custom_name: Some(name),
            custom_wm_class: Some(wm_class),
            custom_icon: None,
            cli_allowed_domains: vec![],
            hide_on_close,
            single_instance,
            tray,
            start_hidden,
            maximize,
            zoom,
            user_agent: None,
            width: None,
            height: None,
            autostart,
            autostart_hidden,
            inject_js,
            inject_css,
        },
    ).ok()?;

    save_app_config(&meta);
    Some(meta)
}

fn execute_install(meta: AppMetadata) {
    let home = dirs::home_dir().expect("Cannot resolve home directory");
    let bin_path = ensure_persistent_binary();

    let apps_dir = home.join(".local/share/applications");
    let icons_dir = home.join(".local/share/icons");
    fs::create_dir_all(&apps_dir).ok();
    fs::create_dir_all(&icons_dir).ok();

    let app_dir = get_profile_dir(&meta.hash);
    fs::create_dir_all(&app_dir).ok();
    let app_icon_path = app_dir.join("icon.png");

    let icon_path = icons_dir.join(format!("appify-{}.png", &meta.hash));

    let presets = get_curated_presets();
    let bundled = presets.iter().find(|p| p.url == meta.url).and_then(|p| p.bundled_icon);

    if let Some(bundled_bytes) = bundled {
        let _ = fs::write(&icon_path, bundled_bytes);
        let _ = fs::write(&app_icon_path, bundled_bytes);
    } else if let Some(ref custom) = meta.custom_icon {
        let p = Path::new(custom);
        if p.exists() {
            if let Ok(bytes) = fs::read(p) {
                if let Some(png) = convert_to_png(&bytes) {
                    let _ = fs::write(&icon_path, &png);
                    let _ = fs::write(&app_icon_path, &png);
                } else {
                    let _ = fs::copy(p, &icon_path);
                    let _ = fs::copy(p, &app_icon_path);
                }
            }
        } else if custom.starts_with("http://") || custom.starts_with("https://") {
            println!("[*] Downloading provided icon URL...");
            fetch_icon(custom, &icon_path);
            let _ = fs::copy(&icon_path, &app_icon_path);
        }
    } else {
        println!("[*] Fetching app icon from website...");
        fetch_icon(&meta.url, &icon_path);
        let _ = fs::copy(&icon_path, &app_icon_path);
    }

    save_app_config(&meta);

    let mut extra_flags = Vec::new();
    if meta.hide_on_close {
        extra_flags.push("--hide-on-close".to_string());
    }
    if meta.single_instance {
        extra_flags.push("--single-instance".to_string());
    }
    if meta.tray {
        extra_flags.push("--tray".to_string());
    }
    if meta.start_hidden && !meta.autostart && !meta.autostart_hidden {
        extra_flags.push("--start-hidden".to_string());
    }
    if meta.maximize {
        extra_flags.push("--maximize".to_string());
    }
    if let Some(z) = meta.zoom {
        extra_flags.push(format!("--zoom {}", z));
    }
    if let Some(ref ua) = meta.user_agent {
        extra_flags.push(format!("--user-agent \"{}\"", ua));
    }
    if let Some(w) = meta.width {
        extra_flags.push(format!("--width {}", w));
    }
    if let Some(h) = meta.height {
        extra_flags.push(format!("--height {}", h));
    }
    for d in &meta.custom_allowed_domains {
        extra_flags.push(format!("--allow-domain \"{}\"", d));
    }

    let extra_args = if !extra_flags.is_empty() {
        format!(" {}", extra_flags.join(" "))
    } else {
        String::new()
    };

    let desktop_path = apps_dir.join(format!("appify-{}.desktop", &meta.hash));
    let desktop_content = format!(
        "[Desktop Entry]\n\
        Version=1.0\n\
        Type=Application\n\
        Name={name}\n\
        Exec={bin} run \"{url}\" \"{name}\" --wm-class \"{wm_class}\" --icon \"{icon}\"{extra_args}\n\
        Icon={icon}\n\
        Terminal=false\n\
        Categories=Network;WebBrowser;\n\
        StartupWMClass={wm_class}\n\
        X-Appify-URL={url}\n\
        X-Appify-Hash={hash}\n",
        name = meta.name,
        bin = bin_path.display(),
        url = meta.url,
        wm_class = meta.wm_class,
        icon = icon_path.display(),
        extra_args = extra_args,
        hash = meta.hash
    );

    let mut file = File::create(&desktop_path).expect("Failed to write desktop file");
    file.write_all(desktop_content.as_bytes()).unwrap();

    let autostart_msg = if meta.autostart || meta.autostart_hidden {
        match install_autostart(&meta, &bin_path, &icon_path, meta.autostart_hidden) {
            Ok(p) => {
                let label = if meta.autostart_hidden {
                    "Enabled (Hidden in tray on boot)"
                } else {
                    "Enabled (Visible on boot)"
                };
                format!("{} [{}]", label, p.display())
            }
            Err(e) => format!("Failed to register ({})", e),
        }
    } else {
        remove_autostart(&meta.hash);
        "Disabled".to_string()
    };

    println!("[+] Successfully installed '{}'!", meta.name);
    println!("    URL:       {}", meta.url);
    println!("    WM_CLASS:  {}", meta.wm_class);
    println!("    Icon:      {}", icon_path.display());
    println!("    Desktop:   {}", desktop_path.display());
    println!("    Autostart: {}", autostart_msg);
}

fn execute_preset_list() {
    let presets = get_curated_presets();
    println!("{:<12} | {:<16} | {:<22} | {:<18} | {}", "Preset ID", "Name", "Category", "WM_CLASS", "URL");
    println!("{}", "-".repeat(95));
    for p in presets {
        let is_installed = resolve_metadata(p.url, RunOptions::default())
            .ok()
            .and_then(|m| load_app_config(&m.hash))
            .is_some();
        let status = if is_installed { " [Installed]" } else { "" };
        println!("{:<12} | {:<16} | {:<22} | {:<18} | {}{}", p.id, p.name, p.category, p.wm_class, p.url, status);
    }
}

fn execute_preset_install(preset_name: &str, mut opts: RunOptions) -> Result<(), String> {
    let presets = get_curated_presets();
    let preset = presets
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case(preset_name))
        .ok_or_else(|| format!("Unknown preset '{}'. Available presets: whatsapp, discord", preset_name))?;

    if opts.custom_name.is_none() {
        opts.custom_name = Some(preset.name.to_string());
    }
    if opts.custom_wm_class.is_none() {
        opts.custom_wm_class = Some(preset.wm_class.to_string());
    }
    if !opts.hide_on_close {
        opts.hide_on_close = preset.default_hide_on_close;
    }
    if !opts.single_instance {
        opts.single_instance = preset.default_single_instance;
    }
    if !opts.tray {
        opts.tray = preset.default_tray;
    }

    let meta = resolve_metadata(preset.url, opts)?;
    execute_install(meta);
    Ok(())
}

fn execute_configure(
    raw_url: &str,
    name: Option<String>,
    wm_class: Option<String>,
    allow_domain: Vec<String>,
    hide_on_close: Option<bool>,
    single_instance: Option<bool>,
    tray: Option<bool>,
    start_hidden: Option<bool>,
    zoom: Option<f64>,
    user_agent: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    autostart: Option<bool>,
    autostart_hidden: Option<bool>,
    inject_js: Option<String>,
    inject_css: Option<String>,
) -> Result<(), String> {
    let base_meta = resolve_metadata(raw_url, RunOptions::default())?;
    let mut config = load_app_config(&base_meta.hash).unwrap_or(base_meta);

    if let Some(n) = name { config.name = n; }
    if let Some(wm) = wm_class { config.wm_class = wm; }
    if !allow_domain.is_empty() { config.custom_allowed_domains = allow_domain; }
    if let Some(hoc) = hide_on_close { config.hide_on_close = hoc; }
    if let Some(si) = single_instance { config.single_instance = si; }
    if let Some(tr) = tray { config.tray = tr; }
    if let Some(sh) = start_hidden { config.start_hidden = sh; }
    if let Some(z) = zoom { config.zoom = Some(z); }
    if let Some(ua) = user_agent { config.user_agent = Some(ua); }
    if let Some(w) = width { config.width = Some(w); }
    if let Some(h) = height { config.height = Some(h); }
    if let Some(as_boot) = autostart { config.autostart = as_boot; }
    if let Some(ash) = autostart_hidden { config.autostart_hidden = ash; }
    if let Some(js) = inject_js { config.inject_js = Some(js); }
    if let Some(css) = inject_css { config.inject_css = Some(css); }

    execute_install(config);
    println!("[+] Successfully reconfigured application.");
    Ok(())
}

fn execute_reinstall(raw_url: &str) -> Result<(), String> {
    let base_meta = resolve_metadata(raw_url, RunOptions::default())?;
    let config = load_app_config(&base_meta.hash).unwrap_or(base_meta);

    // Remove existing icon to force fresh re-fetch
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let icon_p = home.join(format!(".local/share/icons/appify-{}.png", &config.hash));
    let _ = fs::remove_file(&icon_p);

    execute_install(config);
    println!("[+] Successfully reinstalled application and refreshed icon.");
    Ok(())
}

fn execute_clear_cache(raw_url: &str) -> Result<(), String> {
    let meta = resolve_metadata(raw_url, RunOptions::default())?;
    let profile_dir = get_profile_dir(&meta.hash);

    if profile_dir.exists() {
        let config_backup = fs::read_to_string(profile_dir.join("config.json")).ok();
        let js_backup = fs::read_to_string(profile_dir.join("userscript.js")).ok();
        let css_backup = fs::read_to_string(profile_dir.join("userstyle.css")).ok();

        let _ = fs::remove_dir_all(&profile_dir);
        let _ = fs::create_dir_all(&profile_dir);

        if let Some(cfg) = config_backup {
            let _ = fs::write(profile_dir.join("config.json"), cfg);
        }
        if let Some(js) = js_backup {
            let _ = fs::write(profile_dir.join("userscript.js"), js);
        }
        if let Some(css) = css_backup {
            let _ = fs::write(profile_dir.join("userstyle.css"), css);
        }
        println!("[+] Cleared session cache and cookies for '{}'.", meta.name);
    } else {
        println!("[*] No session cache found for '{}'.", meta.name);
    }
    Ok(())
}

fn execute_uninstall(raw_url: &str) {
    let clean = raw_url.trim();
    let meta = if clean.len() == 64 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
        load_app_config(clean)
    } else {
        None
    }
    .or_else(|| resolve_metadata(clean, RunOptions::default()).ok());

    let (hash, name) = match meta {
        Some(m) => (m.hash, m.name),
        None => {
            eprintln!("[!] Could not find or resolve app '{}'", raw_url);
            return;
        }
    };

    let home = dirs::home_dir().unwrap();
    let desktop_path = home.join(format!(".local/share/applications/appify-{}.desktop", &hash));
    let icon_path = home.join(format!(".local/share/icons/appify-{}.png", &hash));
    let profile_dir = dirs::data_local_dir().unwrap().join("appify/profiles").join(&hash);

    let mut removed = false;
    if desktop_path.exists() {
        fs::remove_file(&desktop_path).ok();
        println!("[*] Removed desktop entry.");
        removed = true;
    }
    if icon_path.exists() {
        fs::remove_file(&icon_path).ok();
        println!("[*] Removed icon.");
        removed = true;
    }
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir).ok();
        println!("[*] Removed isolated session cache.");
        removed = true;
    }
    if remove_autostart(&hash) {
        println!("[*] Removed autostart entry.");
        removed = true;
    }

    if removed {
        println!("[+] Successfully uninstalled '{}'", name);
    } else {
        println!("[!] App was not found.");
    }
}

fn execute_list() {
    let home = dirs::home_dir().unwrap();
    let apps_dir = home.join(".local/share/applications");

    println!(
        "{:<22} | {:<18} | {:<15} | {:<8} | {}",
        "App Name", "WM_CLASS", "Autostart", "Scripts", "URL"
    );
    println!("{}", "-".repeat(95));

    if let Ok(entries) = fs::read_dir(apps_dir) {
        let mut apps = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with("appify-") && s.ends_with(".desktop"))
            {
                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let hash = file_stem.trim_start_matches("appify-");
                if let Some(config) = load_app_config(hash) {
                    let autostart_str = get_autostart_status(&config.hash);
                    let p_dir = get_profile_dir(&config.hash);
                    let has_scripts = p_dir.join("userscript.js").exists() || p_dir.join("userstyle.css").exists();
                    let scripts_str = if has_scripts { "Yes" } else { "No" };
                    apps.push((config.name, config.wm_class, autostart_str, scripts_str.to_string(), config.url));
                }
            }
        }
        apps.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, wm, autostart, scripts, url) in apps {
            println!("{:<22} | {:<18} | {:<15} | {:<8} | {}", name, wm, autostart, scripts, url);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct AppInfoDto {
    name: String,
    url: String,
    wm_class: String,
    hash: String,
    icon_path: String,
    icon_base64: Option<String>,
    single_instance: bool,
    hide_on_close: bool,
    tray: bool,
    start_hidden: bool,
    autostart: bool,
    autostart_hidden: bool,
    zoom: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
    custom_allowed_domains: Vec<String>,
    has_custom_scripts: bool,
    #[serde(default)]
    maximize: bool,
    #[serde(default)]
    user_agent: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct PresetInfoDto {
    id: String,
    name: String,
    description: String,
    category: String,
    url: String,
    wm_class: String,
    is_installed: bool,
    installed_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct InstallRequestDto {
    url: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    wm_class: Option<String>,
    #[serde(default)]
    custom_icon: Option<String>,
    #[serde(default)]
    custom_allowed_domains: Vec<String>,
    #[serde(default)]
    hide_on_close: bool,
    #[serde(default)]
    single_instance: bool,
    #[serde(default)]
    tray: bool,
    #[serde(default)]
    start_hidden: bool,
    #[serde(default)]
    maximize: bool,
    #[serde(default)]
    zoom: Option<f64>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    autostart: bool,
    #[serde(default)]
    autostart_hidden: bool,
    #[serde(default)]
    inject_js: Option<String>,
    #[serde(default)]
    inject_css: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct UpdateConfigDto {
    #[serde(default)]
    name: String,
    #[serde(default)]
    wm_class: String,
    #[serde(default)]
    custom_allowed_domains: Vec<String>,
    #[serde(default)]
    hide_on_close: bool,
    #[serde(default)]
    single_instance: bool,
    #[serde(default)]
    tray: bool,
    #[serde(default)]
    start_hidden: bool,
    #[serde(default)]
    maximize: bool,
    #[serde(default)]
    zoom: Option<f64>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    autostart: bool,
    #[serde(default)]
    autostart_hidden: bool,
    #[serde(default)]
    inject_js: Option<String>,
    #[serde(default)]
    inject_css: Option<String>,
    #[serde(default)]
    app_plugins: Option<Vec<AppExtensionRef>>,
    #[serde(default)]
    app_themes: Option<Vec<AppExtensionRef>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ScriptsDto {
    js: String,
    css: String,
    #[serde(default)]
    plugins: Vec<AppExtensionRef>,
    #[serde(default)]
    themes: Vec<AppExtensionRef>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SystemInfoDto {
    bin_path: String,
    profiles_dir: String,
    applications_dir: String,
    autostart_dir: String,
}

#[tauri::command]
fn get_installed_apps() -> Vec<AppInfoDto> {
    let mut list = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let apps_dir = home.join(".local/share/applications");
    let icons_dir = home.join(".local/share/icons");

    if let Ok(entries) = fs::read_dir(apps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with("appify-") && s.ends_with(".desktop"))
            {
                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let hash = file_stem.trim_start_matches("appify-");
                if hash.len() == 64 {
                    if let Some(config) = load_app_config(hash) {
                        let icon_p = icons_dir.join(format!("appify-{}.png", hash));
                        let icon_base64 = if icon_p.exists() {
                            fs::read(&icon_p).ok().map(|b| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &b))
                        } else {
                            None
                        };

                        let p_dir = get_profile_dir(hash);
                        let has_scripts = (p_dir.join("userscript.js").exists() && fs::read_to_string(p_dir.join("userscript.js")).map_or(false, |s| !s.trim().is_empty()))
                            || (p_dir.join("userstyle.css").exists() && fs::read_to_string(p_dir.join("userstyle.css")).map_or(false, |s| !s.trim().is_empty()));

                        list.push(AppInfoDto {
                            name: config.name,
                            url: config.url,
                            wm_class: config.wm_class,
                            hash: config.hash,
                            icon_path: icon_p.display().to_string(),
                            icon_base64,
                            single_instance: config.single_instance,
                            hide_on_close: config.hide_on_close,
                            tray: config.tray,
                            start_hidden: config.start_hidden,
                            autostart: config.autostart,
                            autostart_hidden: config.autostart_hidden,
                            zoom: config.zoom,
                            width: config.width,
                            height: config.height,
                            custom_allowed_domains: config.custom_allowed_domains,
                            has_custom_scripts: has_scripts,
                            maximize: config.maximize,
                            user_agent: config.user_agent,
                        });
                    }
                }
            }
        }
    }
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

#[tauri::command]
fn get_presets() -> Vec<PresetInfoDto> {
    let presets = get_curated_presets();
    let mut result = Vec::new();
    for p in presets {
        let mut is_installed = false;
        let mut installed_hash = None;

        if let Ok(meta) = resolve_metadata(p.url, RunOptions::default()) {
            if let Some(home) = dirs::home_dir() {
                let dp = home.join(format!(".local/share/applications/appify-{}.desktop", meta.hash));
                if dp.exists() {
                    is_installed = true;
                    installed_hash = Some(meta.hash);
                }
            }
        }

        result.push(PresetInfoDto {
            id: p.id.to_string(),
            name: p.name.to_string(),
            description: p.description.to_string(),
            category: p.category.to_string(),
            url: p.url.to_string(),
            wm_class: p.wm_class.to_string(),
            is_installed,
            installed_hash,
        });
    }
    result
}

#[tauri::command]
async fn install_preset_cmd(id: String, options: Option<InstallRequestDto>) -> Result<AppInfoDto, String> {
    let presets = get_curated_presets();
    let preset = presets.into_iter().find(|p| p.id.eq_ignore_ascii_case(&id))
        .ok_or_else(|| format!("Unknown preset: {}", id))?;

    let opts = if let Some(req) = options {
        RunOptions {
            custom_name: req.name.or_else(|| Some(preset.name.to_string())),
            custom_wm_class: req.wm_class.or_else(|| Some(preset.wm_class.to_string())),
            custom_icon: req.custom_icon,
            cli_allowed_domains: req.custom_allowed_domains,
            hide_on_close: req.hide_on_close,
            single_instance: req.single_instance,
            tray: req.tray,
            start_hidden: req.start_hidden,
            maximize: req.maximize,
            zoom: req.zoom,
            user_agent: req.user_agent,
            width: req.width,
            height: req.height,
            autostart: req.autostart,
            autostart_hidden: req.autostart_hidden,
            inject_js: req.inject_js,
            inject_css: req.inject_css,
        }
    } else {
        RunOptions {
            custom_name: Some(preset.name.to_string()),
            custom_wm_class: Some(preset.wm_class.to_string()),
            custom_icon: None,
            cli_allowed_domains: vec![],
            hide_on_close: preset.default_hide_on_close,
            single_instance: preset.default_single_instance,
            tray: preset.default_tray,
            start_hidden: false,
            maximize: false,
            zoom: None,
            user_agent: None,
            width: None,
            height: None,
            autostart: false,
            autostart_hidden: false,
            inject_js: None,
            inject_css: None,
        }
    };

    let meta = resolve_metadata(preset.url, opts)?;
    execute_install(meta.clone());

    let icons_dir = dirs::home_dir().unwrap().join(".local/share/icons");
    let icon_p = icons_dir.join(format!("appify-{}.png", meta.hash));
    let icon_base64 = if icon_p.exists() {
        fs::read(&icon_p).ok().map(|b| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &b))
    } else {
        None
    };

    Ok(AppInfoDto {
        name: meta.name,
        url: meta.url,
        wm_class: meta.wm_class,
        hash: meta.hash,
        icon_path: icon_p.display().to_string(),
        icon_base64,
        single_instance: meta.single_instance,
        hide_on_close: meta.hide_on_close,
        tray: meta.tray,
        start_hidden: meta.start_hidden,
        autostart: meta.autostart,
        autostart_hidden: meta.autostart_hidden,
        zoom: meta.zoom,
        width: meta.width,
        height: meta.height,
        custom_allowed_domains: meta.custom_allowed_domains,
        has_custom_scripts: meta.inject_js.is_some() || meta.inject_css.is_some(),
        maximize: meta.maximize,
        user_agent: meta.user_agent,
    })
}

#[tauri::command]
fn install_custom_cmd(req: InstallRequestDto) -> Result<AppInfoDto, String> {
    let opts = RunOptions {
        custom_name: req.name,
        custom_wm_class: req.wm_class,
        custom_icon: req.custom_icon,
        cli_allowed_domains: req.custom_allowed_domains,
        hide_on_close: req.hide_on_close,
        single_instance: req.single_instance,
        tray: req.tray,
        start_hidden: req.start_hidden,
        maximize: req.maximize,
        zoom: req.zoom,
        user_agent: req.user_agent,
        width: req.width,
        height: req.height,
        autostart: req.autostart,
        autostart_hidden: req.autostart_hidden,
        inject_js: req.inject_js,
        inject_css: req.inject_css,
    };

    let meta = resolve_metadata(&req.url, opts)?;
    execute_install(meta.clone());

    let icons_dir = dirs::home_dir().unwrap().join(".local/share/icons");
    let icon_p = icons_dir.join(format!("appify-{}.png", meta.hash));
    let icon_base64 = if icon_p.exists() {
        fs::read(&icon_p).ok().map(|b| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &b))
    } else {
        None
    };

    Ok(AppInfoDto {
        name: meta.name,
        url: meta.url,
        wm_class: meta.wm_class,
        hash: meta.hash,
        icon_path: icon_p.display().to_string(),
        icon_base64,
        single_instance: meta.single_instance,
        hide_on_close: meta.hide_on_close,
        tray: meta.tray,
        start_hidden: meta.start_hidden,
        autostart: meta.autostart,
        autostart_hidden: meta.autostart_hidden,
        zoom: meta.zoom,
        width: meta.width,
        height: meta.height,
        custom_allowed_domains: meta.custom_allowed_domains,
        has_custom_scripts: meta.inject_js.is_some() || meta.inject_css.is_some(),
        maximize: meta.maximize,
        user_agent: meta.user_agent,
    })
}

#[tauri::command]
fn update_app_config_cmd(hash: String, req: UpdateConfigDto, reinstall_icon: Option<bool>) -> Result<AppInfoDto, String> {
    let mut config = load_app_config(&hash).ok_or_else(|| "App not found".to_string())?;

    config.name = req.name;
    config.wm_class = req.wm_class;
    config.custom_allowed_domains = req.custom_allowed_domains;
    config.hide_on_close = req.hide_on_close;
    config.single_instance = req.single_instance;
    config.tray = req.tray;
    config.start_hidden = req.start_hidden;
    config.maximize = req.maximize;
    config.zoom = req.zoom;
    config.user_agent = req.user_agent;
    config.width = req.width;
    config.height = req.height;
    config.autostart = req.autostart;
    config.autostart_hidden = req.autostart_hidden;

    if let Some(p) = req.app_plugins {
        config.app_plugins = p;
    }
    if let Some(t) = req.app_themes {
        config.app_themes = t;
    }

    if let Some(js) = req.inject_js {
        config.inject_js = Some(js);
    }
    if let Some(css) = req.inject_css {
        config.inject_css = Some(css);
    }

    if reinstall_icon.unwrap_or(false) {
        let home = dirs::home_dir().unwrap();
        let icon_path = home.join(format!(".local/share/icons/appify-{}.png", &config.hash));
        let _ = fs::remove_file(&icon_path);
    }

    execute_install(config.clone());

    let icons_dir = dirs::home_dir().unwrap().join(".local/share/icons");
    let icon_p = icons_dir.join(format!("appify-{}.png", config.hash));
    let icon_base64 = if icon_p.exists() {
        fs::read(&icon_p).ok().map(|b| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &b))
    } else {
        None
    };

    Ok(AppInfoDto {
        name: config.name,
        url: config.url,
        wm_class: config.wm_class,
        hash: config.hash,
        icon_path: icon_p.display().to_string(),
        icon_base64,
        single_instance: config.single_instance,
        hide_on_close: config.hide_on_close,
        tray: config.tray,
        start_hidden: config.start_hidden,
        autostart: config.autostart,
        autostart_hidden: config.autostart_hidden,
        zoom: config.zoom,
        width: config.width,
        height: config.height,
        custom_allowed_domains: config.custom_allowed_domains,
        has_custom_scripts: config.inject_js.is_some() || config.inject_css.is_some() || !config.app_plugins.is_empty() || !config.app_themes.is_empty(),
        maximize: config.maximize,
        user_agent: config.user_agent,
    })
}

#[tauri::command]
fn uninstall_app_cmd(hash: String) -> Result<(), String> {
    execute_uninstall(&hash);
    Ok(())
}

#[tauri::command]
fn launch_app_cmd(hash: String) -> Result<(), String> {
    let config = load_app_config(&hash).ok_or_else(|| "App not found".to_string())?;
    let bin = ensure_persistent_binary();
    std::process::Command::new(bin)
        .arg("run")
        .arg(&config.url)
        .spawn()
        .map_err(|e| format!("Failed to spawn app process: {}", e))?;
    Ok(())
}

#[tauri::command]
fn clear_app_cache_cmd(hash: String) -> Result<(), String> {
    let config = load_app_config(&hash).ok_or_else(|| "App not found".to_string())?;
    execute_clear_cache(&config.url)
}

#[tauri::command]
fn get_registry_cmd() -> ExtensionRegistry {
    load_registry()
}

#[tauri::command]
fn save_registry_item_cmd(item: ExtensionItem) -> Result<(), String> {
    let mut reg = load_registry();
    if item.category == "plugin" {
        if let Some(existing) = reg.plugins.iter_mut().find(|p| p.id == item.id) {
            if existing.is_builtin {
                return Err("Built-in plugins cannot be modified in the global registry.".to_string());
            }
            *existing = item;
        } else {
            let mut new_item = item;
            new_item.is_builtin = false;
            reg.plugins.push(new_item);
        }
    } else {
        if let Some(existing) = reg.themes.iter_mut().find(|t| t.id == item.id) {
            if existing.is_builtin {
                return Err("Built-in themes cannot be modified in the global registry.".to_string());
            }
            *existing = item;
        } else {
            let mut new_item = item;
            new_item.is_builtin = false;
            reg.themes.push(new_item);
        }
    }
    save_registry(&reg)
}

#[tauri::command]
fn delete_registry_item_cmd(id: String) -> Result<(), String> {
    let mut reg = load_registry();
    if let Some(pos) = reg.plugins.iter().position(|p| p.id == id) {
        if reg.plugins[pos].is_builtin {
            return Err("Cannot delete built-in plugins from registry.".to_string());
        }
        reg.plugins.remove(pos);
        return save_registry(&reg);
    }
    if let Some(pos) = reg.themes.iter().position(|t| t.id == id) {
        if reg.themes[pos].is_builtin {
            return Err("Cannot delete built-in themes from registry.".to_string());
        }
        reg.themes.remove(pos);
        return save_registry(&reg);
    }
    Err("Extension item not found".to_string())
}

#[tauri::command]
fn get_marketplace_catalog_cmd() -> Result<MarketplaceCatalogDto, String> {
    fetch_marketplace_catalog()
}

#[tauri::command]
fn install_marketplace_item_cmd(id: String) -> Result<ExtensionItem, String> {
    install_marketplace_item(&id)
}

fn execute_marketplace_list() {
    println!("\n🌐 Appify Community Marketplace (cdn.appify.aerovex.net)\n");
    println!("{:<24} | {:<8} | {:<7} | {:<16} | {}", "ID", "TYPE", "VER", "AUTHOR", "DESCRIPTION");
    println!("{:-<24}-+-{:-<8}-+-{:-<7}-+-{:-<16}-+-{:-<35}", "", "", "", "", "");

    match fetch_marketplace_catalog() {
        Ok(catalog) => {
            let registry = load_registry();
            let is_installed = |id: &str| {
                registry.plugins.iter().any(|p| p.id == id) || registry.themes.iter().any(|t| t.id == id)
            };

            for item in catalog.items {
                let status = if is_installed(&item.id) { "[Installed]" } else { "" };
                let type_str = if item.category == "plugin" { "Plugin" } else { "Theme" };
                let desc = if item.description.len() > 40 {
                    format!("{}...", &item.description[..37])
                } else {
                    item.description.clone()
                };
                println!("{:<24} | {:<8} | {:<7} | {:<16} | {} {}", item.id, type_str, item.version, item.author, desc, status);
            }
            println!("\nTotal: {} items available ({} plugins, {} themes)", catalog.total_items, catalog.plugins_count, catalog.themes_count);
            println!("Install with: appify marketplace install <id>\n");
        }
        Err(e) => {
            eprintln!("[!] Failed to fetch marketplace catalog: {}", e);
        }
    }
}

fn execute_marketplace_search(query: &str) {
    let q = query.to_lowercase();
    println!("\n🔍 Searching Marketplace for '{}'...\n", query);
    println!("{:<24} | {:<8} | {:<7} | {:<16} | {}", "ID", "TYPE", "VER", "AUTHOR", "DESCRIPTION");
    println!("{:-<24}-+-{:-<8}-+-{:-<7}-+-{:-<16}-+-{:-<35}", "", "", "", "", "");

    match fetch_marketplace_catalog() {
        Ok(catalog) => {
            let mut matches = 0;
            for item in catalog.items {
                if item.id.to_lowercase().contains(&q)
                    || item.name.to_lowercase().contains(&q)
                    || item.description.to_lowercase().contains(&q)
                    || item.tags.iter().any(|t| t.to_lowercase().contains(&q))
                {
                    matches += 1;
                    let type_str = if item.category == "plugin" { "Plugin" } else { "Theme" };
                    let desc = if item.description.len() > 40 {
                        format!("{}...", &item.description[..37])
                    } else {
                        item.description.clone()
                    };
                    println!("{:<24} | {:<8} | {:<7} | {:<16} | {}", item.id, type_str, item.version, item.author, desc);
                }
            }
            if matches == 0 {
                println!("No extensions matched '{}'.", query);
            } else {
                println!("\nFound {} matching extensions.", matches);
                println!("Install with: appify marketplace install <id>\n");
            }
        }
        Err(e) => eprintln!("[!] Failed to search marketplace: {}", e),
    }
}

fn execute_marketplace_install(id: &str) -> Result<(), String> {
    println!("⬇️  Installing extension '{}' from marketplace...", id);
    match install_marketplace_item(id) {
        Ok(item) => {
            println!("✓ Successfully installed '{}' ({}) into registry!", item.name, item.category);
            println!("  You can now attach it to any app in the Extension Studio or App Options.");
            Ok(())
        }
        Err(e) => {
            eprintln!("[!] Installation failed: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
fn get_app_scripts_cmd(hash: String) -> Result<ScriptsDto, String> {
    let p_dir = get_profile_dir(&hash);
    let js = fs::read_to_string(p_dir.join("userscript.js")).unwrap_or_default();
    let css = fs::read_to_string(p_dir.join("userstyle.css")).unwrap_or_default();
    let config = load_app_config(&hash);
    let mut plugins = config.as_ref().map(|c| c.app_plugins.clone()).unwrap_or_default();
    let mut themes = config.as_ref().map(|c| c.app_themes.clone()).unwrap_or_default();

    if plugins.is_empty() && !js.trim().is_empty() {
        if js.contains("cleanWhatsAppDownloadPromos") {
            plugins.push(AppExtensionRef {
                id: "builtin-wa-promo-cleaner".to_string(),
                name: "WhatsApp Promo Cleaner".to_string(),
                category: "plugin".to_string(),
                enabled: true,
                custom_content: None,
            });
        } else {
            plugins.push(AppExtensionRef {
                id: format!("custom-js-{}", hash),
                name: "Custom JavaScript".to_string(),
                category: "plugin".to_string(),
                enabled: true,
                custom_content: Some(js.clone()),
            });
        }
    }

    if themes.is_empty() && !css.trim().is_empty() {
        if css.contains("Get WhatsApp") || css.contains("whatsapp.com/download") {
            themes.push(AppExtensionRef {
                id: "builtin-wa-promo-css".to_string(),
                name: "WhatsApp Clean Layout".to_string(),
                category: "theme".to_string(),
                enabled: true,
                custom_content: None,
            });
        } else {
            themes.push(AppExtensionRef {
                id: format!("custom-css-{}", hash),
                name: "Custom CSS".to_string(),
                category: "theme".to_string(),
                enabled: true,
                custom_content: Some(css.clone()),
            });
        }
    }

    Ok(ScriptsDto { js, css, plugins, themes })
}

#[tauri::command]
fn save_app_scripts_cmd(
    hash: String,
    js: String,
    css: String,
    plugins: Option<Vec<AppExtensionRef>>,
    themes: Option<Vec<AppExtensionRef>>,
) -> Result<(), String> {
    if let Some(mut config) = load_app_config(&hash) {
        if let Some(p) = plugins {
            config.app_plugins = p;
        }
        if let Some(t) = themes {
            config.app_themes = t;
        }
        if !js.trim().is_empty() {
            config.inject_js = Some(js);
        }
        if !css.trim().is_empty() {
            config.inject_css = Some(css);
        }
        save_app_config(&config);
    } else {
        let p_dir = get_profile_dir(&hash);
        fs::create_dir_all(&p_dir).map_err(|e| e.to_string())?;
        fs::write(p_dir.join("userscript.js"), js).map_err(|e| e.to_string())?;
        fs::write(p_dir.join("userstyle.css"), css).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_app_folder_cmd(hash: String) -> Result<(), String> {
    let p_dir = get_profile_dir(&hash);
    if p_dir.exists() {
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(&p_dir).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(&p_dir).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(&p_dir).spawn();
        }
        Ok(())
    } else {
        Err("App folder not found".to_string())
    }
}

#[tauri::command]
fn get_system_info_cmd() -> SystemInfoDto {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    SystemInfoDto {
        bin_path: ensure_persistent_binary().display().to_string(),
        profiles_dir: dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("appify").display().to_string(),
        applications_dir: home.join(".local/share/applications").display().to_string(),
        autostart_dir: dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("autostart").display().to_string(),
    }
}

#[tauri::command]
fn self_install_binary_cmd() -> Result<String, String> {
    let path = execute_self_install();
    Ok(format!("Appify registered successfully at {}", path.display()))
}

fn execute_gui_manager() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            get_presets,
            install_preset_cmd,
            install_custom_cmd,
            update_app_config_cmd,
            uninstall_app_cmd,
            launch_app_cmd,
            clear_app_cache_cmd,
            get_app_scripts_cmd,
            save_app_scripts_cmd,
            get_system_info_cmd,
            self_install_binary_cmd,
            open_app_folder_cmd,
            get_registry_cmd,
            save_registry_item_cmd,
            delete_registry_item_cmd,
            get_marketplace_catalog_cmd,
            install_marketplace_item_cmd,
        ])
        .setup(|app| {
            let icon_img = app.default_window_icon().cloned()
                .or_else(|| tauri::image::Image::from_bytes(FALLBACK_ICON_BYTES).ok());

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "manager",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Appify — Desktop Web App Manager")
            .inner_size(1120.0, 750.0)
            .min_inner_size(860.0, 580.0)
            .center()
            .disable_drag_drop_handler()
            .initialization_script(r#"
                document.addEventListener('contextmenu', function(e) { e.preventDefault(); return false; }, true);
                document.addEventListener('dragstart', function(e) {
                    if (e.target.tagName === 'IMG' || e.target.closest('.app-card') || e.target.closest('.preset-card')) {
                        e.preventDefault();
                    }
                }, true);
            "#);

            if let Some(icon) = icon_img {
                builder = builder.icon(icon)?;
            }

            let _win = builder.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Appify Manager");
}

fn main() {
    migrate_legacy_profiles();
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Ui) | Some(Commands::Gui) => {
            execute_gui_manager();
        }
        Some(Commands::Install {
            url,
            name,
            wm_class,
            icon,
            allow_domain,
            hide_on_close,
            single_instance,
            tray,
            start_hidden,
            maximize,
            zoom,
            user_agent,
            width,
            height,
            autostart,
            autostart_hidden,
            inject_js,
            inject_css,
        }) => {
            let opts = RunOptions {
                custom_name: name,
                custom_wm_class: wm_class,
                custom_icon: icon,
                cli_allowed_domains: allow_domain,
                hide_on_close,
                single_instance,
                tray,
                start_hidden,
                maximize,
                zoom,
                user_agent,
                width,
                height,
                autostart,
                autostart_hidden,
                inject_js,
                inject_css,
            };
            match resolve_metadata(&url, opts) {
                Ok(meta) => execute_install(meta),
                Err(e) => eprintln!("[!] Error: {}", e),
            }
        }
        Some(Commands::Run {
            url,
            name,
            wm_class,
            icon,
            allow_domain,
            hide_on_close,
            single_instance,
            tray,
            start_hidden,
            maximize,
            zoom,
            user_agent,
            width,
            height,
            inject_js,
            inject_css,
        }) => {
            let opts = RunOptions {
                custom_name: name,
                custom_wm_class: wm_class,
                custom_icon: icon,
                cli_allowed_domains: allow_domain,
                hide_on_close,
                single_instance,
                tray,
                start_hidden,
                maximize,
                zoom,
                user_agent,
                width,
                height,
                autostart: false,
                autostart_hidden: false,
                inject_js,
                inject_css,
            };
            match resolve_metadata(&url, opts) {
                Ok(meta) => execute_run(meta),
                Err(e) => eprintln!("[!] Error: {}", e),
            }
        }
        Some(Commands::Preset { action }) => match action {
            PresetCommands::List => {
                execute_preset_list();
            }
            PresetCommands::Install {
                name,
                name_override,
                wm_class,
                icon,
                allow_domain,
                hide_on_close,
                single_instance,
                tray,
                start_hidden,
                zoom,
                autostart,
                autostart_hidden,
                inject_js,
                inject_css,
            } => {
                let opts = RunOptions {
                    custom_name: name_override,
                    custom_wm_class: wm_class,
                    custom_icon: icon,
                    cli_allowed_domains: allow_domain,
                    hide_on_close,
                    single_instance,
                    tray,
                    start_hidden,
                    maximize: false,
                    zoom,
                    user_agent: None,
                    width: None,
                    height: None,
                    autostart,
                    autostart_hidden,
                    inject_js,
                    inject_css,
                };
                if let Err(e) = execute_preset_install(&name, opts) {
                    eprintln!("[!] Error: {}", e);
                }
            }
        },
        Some(Commands::Configure {
            url,
            name,
            wm_class,
            allow_domain,
            hide_on_close,
            single_instance,
            tray,
            start_hidden,
            zoom,
            user_agent,
            width,
            height,
            autostart,
            autostart_hidden,
            inject_js,
            inject_css,
        }) => {
            if let Err(e) = execute_configure(
                &url,
                name,
                wm_class,
                allow_domain,
                hide_on_close,
                single_instance,
                tray,
                start_hidden,
                zoom,
                user_agent,
                width,
                height,
                autostart,
                autostart_hidden,
                inject_js,
                inject_css,
            ) {
                eprintln!("[!] Error: {}", e);
            }
        }
        Some(Commands::Reinstall { url }) => {
            if let Err(e) = execute_reinstall(&url) {
                eprintln!("[!] Error: {}", e);
            }
        }
        Some(Commands::ClearCache { url }) => {
            if let Err(e) = execute_clear_cache(&url) {
                eprintln!("[!] Error: {}", e);
            }
        }
        Some(Commands::Uninstall { url }) => {
            execute_uninstall(&url);
        }
        Some(Commands::List) => {
            execute_list();
        }
        Some(Commands::SelfInstall) => {
            execute_self_install();
        }
        Some(Commands::Marketplace { action }) => match action {
            MarketplaceCommands::List => {
                execute_marketplace_list();
            }
            MarketplaceCommands::Search { query } => {
                execute_marketplace_search(&query);
            }
            MarketplaceCommands::Install { id } => {
                let _ = execute_marketplace_install(&id);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popular_aliases_exist() {
        let aliases = get_popular_aliases();
        assert!(aliases.contains_key("whatsapp"));
        assert!(aliases.contains_key("discord"));
        assert!(aliases.contains_key("telegram"));
        assert!(aliases.contains_key("spotify"));
        assert!(aliases.contains_key("netflix"));
        assert!(aliases.contains_key("youtube"));
        assert!(aliases.contains_key("twitter"));
        assert!(aliases.contains_key("x"));
        assert!(aliases.contains_key("reddit"));
        assert!(aliases.contains_key("chatgpt"));
        assert!(aliases.contains_key("notion"));
        assert!(aliases.contains_key("figma"));
        assert!(aliases.contains_key("gmail"));

        for (alias, info) in &aliases {
            assert!(!alias.is_empty());
            assert!(info.url.starts_with("https://"));
            assert!(!info.name.is_empty());
            assert!(!info.wm_class.is_empty());
        }
    }

    #[test]
    fn test_resolve_metadata_alias() {
        let meta = resolve_metadata("whatsapp", RunOptions::default()).unwrap();
        assert_eq!(meta.url, "https://web.whatsapp.com");
        assert_eq!(meta.name, "WhatsApp");
        assert_eq!(meta.wm_class, "whatsapp-desktop");
        assert_eq!(meta.base_domain, "whatsapp.com");
        assert_eq!(meta.hash.len(), 64);
        assert!(meta.allowed_domains.contains(&"whatsapp.net".to_string()));
        assert!(meta.allowed_domains.contains(&"fbcdn.net".to_string()));
    }

    #[test]
    fn test_resolve_metadata_case_insensitive_alias() {
        let meta = resolve_metadata("  DISCORD  ", RunOptions::default()).unwrap();
        assert_eq!(meta.url, "https://discord.com/app");
        assert_eq!(meta.name, "Discord");
        assert_eq!(meta.wm_class, "discord-app");
        assert!(meta.allowed_domains.contains(&"discord.gg".to_string()));
    }

    #[test]
    fn test_resolve_metadata_raw_url_no_scheme() {
        let meta = resolve_metadata("github.com", RunOptions::default()).unwrap();
        assert_eq!(meta.url, "https://github.com");
        assert_eq!(meta.name, "github.com");
        assert_eq!(meta.base_domain, "github.com");
        assert!(meta.wm_class.starts_with("appify-"));
    }

    #[test]
    fn test_resolve_metadata_trailing_slash_stripped() {
        let meta = resolve_metadata("https://example.org/", RunOptions::default()).unwrap();
        assert_eq!(meta.url, "https://example.org");
    }

    #[test]
    fn test_resolve_metadata_custom_overrides() {
        let meta = resolve_metadata(
            "https://linear.app",
            RunOptions {
                custom_name: Some("Linear Work".to_string()),
                custom_wm_class: Some("custom-linear".to_string()),
                custom_icon: Some("/path/to/icon.png".to_string()),
                cli_allowed_domains: vec!["linear.com".to_string()],
                hide_on_close: true,
                single_instance: true,
                tray: true,
                start_hidden: false,
                maximize: true,
                zoom: Some(1.2),
                user_agent: Some("CustomUA/1.0".to_string()),
                width: Some(1280.0),
                height: Some(900.0),
                autostart: false,
                autostart_hidden: false,
                inject_js: None,
                inject_css: None,
            },
        )
        .unwrap();

        assert_eq!(meta.name, "Linear Work");
        assert_eq!(meta.wm_class, "custom-linear");
        assert_eq!(meta.custom_icon, Some("/path/to/icon.png".to_string()));
        assert!(meta.allowed_domains.contains(&"linear.com".to_string()));
        assert_eq!(meta.custom_allowed_domains, vec!["linear.com".to_string()]);
        assert!(meta.hide_on_close);
        assert!(meta.single_instance);
        assert!(meta.tray);
        assert!(meta.maximize);
        assert_eq!(meta.zoom, Some(1.2));
        assert_eq!(meta.user_agent, Some("CustomUA/1.0".to_string()));
        assert_eq!(meta.width, Some(1280.0));
        assert_eq!(meta.height, Some(900.0));
    }

    #[test]
    fn test_resolve_metadata_auto_enables_tray_on_hide_on_close() {
        let meta = resolve_metadata(
            "whatsapp",
            RunOptions {
                hide_on_close: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(meta.hide_on_close);
        assert!(meta.tray);
    }

    #[test]
    fn test_resolve_metadata_localhost() {
        let meta = resolve_metadata("http://localhost:3000", RunOptions::default()).unwrap();
        assert_eq!(meta.url, "http://localhost:3000");
        assert_eq!(meta.base_domain, "localhost");
    }

    #[test]
    fn test_resolve_metadata_invalid_domain() {
        let res = resolve_metadata("notadomain", RunOptions::default());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not appear to be a valid domain"));
    }

    #[test]
    fn test_hash_consistency() {
        let meta1 = resolve_metadata("https://github.com", RunOptions::default()).unwrap();
        let meta2 = resolve_metadata("github.com", RunOptions::default()).unwrap();
        assert_eq!(meta1.hash, meta2.hash);
    }

    #[test]
    fn test_domain_matches() {
        assert!(domain_matches("flows.whatsapp.net", "whatsapp.net"));
        assert!(domain_matches("webtp.whatsapp.net", "whatsapp.net"));
        assert!(domain_matches("whatsapp.net", "whatsapp.net"));
        assert!(domain_matches("static.whatsapp.net", "*.whatsapp.net"));
        assert!(domain_matches("static.whatsapp.net", ".whatsapp.net"));
        assert!(domain_matches("WEB.WHATSAPP.COM", "whatsapp.com"));
        assert!(!domain_matches("attackerwhatsapp.net", "whatsapp.net"));
        assert!(!domain_matches("google.com", "whatsapp.net"));
    }

    #[test]
    fn test_is_internal_navigation_whatsapp() {
        let allowed = vec![
            "whatsapp.com".to_string(),
            "whatsapp.net".to_string(),
            "fbcdn.net".to_string(),
            "accounts.google.com".to_string(),
        ];
        let base = "whatsapp.com";

        let url_flows = Url::parse("https://flows.whatsapp.net/flows/cache_management/").unwrap();
        assert!(is_internal_navigation(&url_flows, base, &allowed));

        let url_pdf = Url::parse("https://webtp.whatsapp.net/pdf-viewer/?locale=en_GB").unwrap();
        assert!(is_internal_navigation(&url_pdf, base, &allowed));

        let url_main = Url::parse("https://web.whatsapp.com").unwrap();
        assert!(is_internal_navigation(&url_main, base, &allowed));

        let url_sso = Url::parse("https://accounts.google.com/o/oauth2/v2/auth?client_id=123").unwrap();
        assert!(is_internal_navigation(&url_sso, base, &allowed));

        let url_external = Url::parse("https://nytimes.com/article123").unwrap();
        assert!(!is_internal_navigation(&url_external, base, &allowed));
    }

    #[test]
    fn test_is_internal_navigation_schemes() {
        let allowed = vec![];
        let base = "example.com";

        let blob_url = Url::parse("blob:https://example.com/1234-5678").unwrap();
        assert!(is_internal_navigation(&blob_url, base, &allowed));

        let data_url = Url::parse("data:image/png;base64,iVBORw0KGgo=").unwrap();
        assert!(is_internal_navigation(&data_url, base, &allowed));

        let about_url = Url::parse("about:blank").unwrap();
        assert!(is_internal_navigation(&about_url, base, &allowed));
    }

    #[test]
    fn test_persistent_bin_path() {
        let path = get_persistent_bin_path();
        #[cfg(unix)]
        assert!(path.ends_with(".local/bin/appify"));
        #[cfg(windows)]
        assert!(path.ends_with("appify.exe"));
    }

    #[test]
    fn test_copy_atomic() {
        let temp_dir = std::env::temp_dir().join("appify_test_atomic");
        let src = temp_dir.join("src_file.txt");
        let dst = temp_dir.join("subdir").join("dst_file.txt");

        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&src, b"hello appify").unwrap();

        copy_atomic(&src, &dst).unwrap();
        assert!(dst.exists());
        assert_eq!(fs::read(&dst).unwrap(), b"hello appify");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_convert_to_png_with_fallback() {
        let converted = convert_to_png(FALLBACK_ICON_BYTES);
        assert!(converted.is_some());
        let png = converted.unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn test_resolve_metadata_autostart_hidden() {
        let meta = resolve_metadata(
            "whatsapp",
            RunOptions {
                autostart_hidden: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(meta.autostart);
        assert!(meta.autostart_hidden);
        assert!(meta.tray);
    }

    #[test]
    fn test_resolve_metadata_autostart_with_start_hidden() {
        let meta = resolve_metadata(
            "whatsapp",
            RunOptions {
                autostart: true,
                start_hidden: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(meta.autostart);
        assert!(meta.autostart_hidden);
        assert!(meta.tray);
    }

    #[test]
    fn test_browser_integration_script() {
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("Notification"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("clipboard-read"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("clipboard-write"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("microphone"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("camera"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("navigator.permissions.query"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("navigator.clipboard.read"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("__appify_dispatch_paste_image"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("toggle_devtools"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("check_clipboard_paste"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("F12"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("__appify_handle_download_url"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("download_request"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("download_chunk"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("__appify_download_start_transfer"));
        assert!(BROWSER_INTEGRATION_SCRIPT.contains("__appify_download_cancel"));
    }

    #[test]
    fn test_whatsapp_promo_cleaner_does_not_hide_media_downloads() {
        let presets = get_curated_presets();
        let wa = presets.iter().find(|p| p.id == "whatsapp").expect("WhatsApp preset must exist");
        let css = wa.default_inject_css.as_deref().unwrap_or("");
        let js = wa.default_inject_js.as_deref().unwrap_or("");

        // Crucial: Must NOT contain blanket download selectors that hide chat/media download buttons
        assert!(!css.contains("[data-testid*=\"download\""), "CSS should not hide media download buttons");
        assert!(!css.contains("a[href*=\"/download\"]"), "CSS should not blanket hide /download links");
        assert!(!js.contains("'a[href*=\"download\"]'"), "JS should not blanket hide download anchors");

        // Must still hide desktop app promo banners
        assert!(css.contains("whatsapp.com/download"));
        assert!(css.contains("data-testid*=\"intro-banner\""));
        assert!(js.contains("cleanWhatsAppDownloadPromos"));

        // Verify builtin extensions as well
        let builtins = get_builtin_extensions();
        let plugin = builtins.plugins.iter().find(|p| p.id == "builtin-wa-promo-cleaner").expect("wa cleaner plugin exists");
        assert!(!plugin.content.contains("'a[href*=\"download\"]'"));

        let theme = builtins.themes.iter().find(|t| t.id == "builtin-wa-promo-css").expect("wa cleaner css exists");
        assert!(!theme.content.contains("[data-testid*=\"download\""));
        assert!(!theme.content.contains("a[href*=\"/download\"]"));
    }

    #[test]
    fn test_clipboard_image_function() {
        // Calling get_clipboard_image_png directly should not panic
        let _ = get_clipboard_image_png();
    }

    #[test]
    fn test_convert_to_png_scored_square_and_rectangular() {
        // 1. Square image (64x64)
        let img_square = image::RgbaImage::new(64, 64);
        let mut buf_sq = Vec::new();
        img_square
            .write_to(&mut std::io::Cursor::new(&mut buf_sq), image::ImageFormat::Png)
            .unwrap();

        let (out_sq, is_sq) = convert_to_png_scored(&buf_sq).unwrap();
        assert!(is_sq, "64x64 should be scored as square");
        let decoded_sq = image::load_from_memory(&out_sq).unwrap();
        assert_eq!(decoded_sq.width(), 64);
        assert_eq!(decoded_sq.height(), 64);

        // 2. Wide rectangular banner (1200x630, e.g. og:image)
        let img_rect = image::RgbaImage::new(1200, 630);
        let mut buf_rect = Vec::new();
        img_rect
            .write_to(&mut std::io::Cursor::new(&mut buf_rect), image::ImageFormat::Png)
            .unwrap();

        let (out_rect, is_rect_sq) = convert_to_png_scored(&buf_rect).unwrap();
        assert!(!is_rect_sq, "1200x630 banner should not be scored as square icon");
        let decoded_rect = image::load_from_memory(&out_rect).unwrap();
        // Should be center-cropped to 630x630 square
        assert_eq!(decoded_rect.width(), 630);
        assert_eq!(decoded_rect.height(), 630);
    }

    #[test]
    fn test_curated_presets_all() {
        let presets = get_curated_presets();
        assert_eq!(presets.len(), 4, "Should have WhatsApp, Discord, Telegram, and Spotify");

        let whatsapp = presets.iter().find(|p| p.id == "whatsapp").expect("WhatsApp preset missing");
        assert_eq!(whatsapp.name, "WhatsApp");
        assert_eq!(whatsapp.url, "https://web.whatsapp.com");
        assert_eq!(whatsapp.wm_class, "whatsapp-desktop");
        assert!(whatsapp.default_hide_on_close);
        assert!(whatsapp.default_single_instance);
        assert!(whatsapp.default_tray);
        assert!(whatsapp.ecosystem_domains.contains(&"whatsapp.net"));
        assert!(whatsapp.ecosystem_domains.contains(&"fbcdn.net"));

        let discord = presets.iter().find(|p| p.id == "discord").expect("Discord preset missing");
        assert_eq!(discord.name, "Discord");
        assert_eq!(discord.url, "https://discord.com/app");
        assert_eq!(discord.wm_class, "discord-app");
        assert!(discord.default_hide_on_close);
        assert!(discord.default_single_instance);
        assert!(discord.default_tray);
        assert!(discord.ecosystem_domains.contains(&"discord.gg"));
        assert!(discord.ecosystem_domains.contains(&"discordapp.com"));

        let telegram = presets.iter().find(|p| p.id == "telegram").expect("Telegram preset missing");
        assert_eq!(telegram.name, "Telegram");
        assert_eq!(telegram.url, "https://web.telegram.org");
        assert_eq!(telegram.wm_class, "telegram-web");
        assert!(telegram.default_hide_on_close);
        assert!(telegram.default_single_instance);
        assert!(telegram.default_tray);
        assert!(telegram.ecosystem_domains.contains(&"telegram.org"));
        assert!(telegram.ecosystem_domains.contains(&"t.me"));

        let spotify = presets.iter().find(|p| p.id == "spotify").expect("Spotify preset missing");
        assert_eq!(spotify.name, "Spotify");
        assert_eq!(spotify.url, "https://open.spotify.com");
        assert_eq!(spotify.wm_class, "spotify-web");
        assert!(spotify.default_hide_on_close);
        assert!(spotify.default_single_instance);
        assert!(spotify.default_tray);
        assert!(spotify.ecosystem_domains.contains(&"spotify.com"));
        assert!(spotify.ecosystem_domains.contains(&"scdn.co"));
    }

    #[test]
    fn test_resolve_metadata_preset_priority() {
        let meta_wa = resolve_metadata("whatsapp", RunOptions::default()).unwrap();
        assert_eq!(meta_wa.name, "WhatsApp");
        assert_eq!(meta_wa.url, "https://web.whatsapp.com");
        assert_eq!(meta_wa.wm_class, "whatsapp-desktop");

        let meta_dc = resolve_metadata("discord", RunOptions::default()).unwrap();
        assert_eq!(meta_dc.name, "Discord");
        assert_eq!(meta_dc.url, "https://discord.com/app");
        assert_eq!(meta_dc.wm_class, "discord-app");

        let meta_tg = resolve_metadata("telegram", RunOptions::default()).unwrap();
        assert_eq!(meta_tg.name, "Telegram");
        assert_eq!(meta_tg.url, "https://web.telegram.org");
        assert_eq!(meta_tg.wm_class, "telegram-web");

        let meta_sp = resolve_metadata("spotify", RunOptions::default()).unwrap();
        assert_eq!(meta_sp.name, "Spotify");
        assert_eq!(meta_sp.url, "https://open.spotify.com");
        assert_eq!(meta_sp.wm_class, "spotify-web");
    }

    #[test]
    fn test_resolve_script_content_inline_and_file() {
        assert_eq!(resolve_script_content(None), None);
        assert_eq!(
            resolve_script_content(Some("console.log('hi');".to_string())),
            Some("console.log('hi');".to_string())
        );

        // Test reading from file
        let tmp = std::env::temp_dir().join("appify_test_script.js");
        fs::write(&tmp, "var test = 42;").unwrap();
        let resolved = resolve_script_content(Some(tmp.display().to_string()));
        let _ = fs::remove_file(&tmp);
        assert_eq!(resolved, Some("var test = 42;".to_string()));
    }

    #[test]
    fn test_app_metadata_json_roundtrip() {
        let meta = resolve_metadata(
            "https://linear.app",
            RunOptions {
                custom_name: Some("Linear App".to_string()),
                inject_js: Some("alert('hello');".to_string()),
                inject_css: Some("body { background: black; }".to_string()),
                hide_on_close: true,
                single_instance: true,
                tray: true,
                ..Default::default()
            },
        )
        .unwrap();

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: AppMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Linear App");
        assert_eq!(deserialized.url, "https://linear.app");
        assert_eq!(deserialized.inject_js, Some("alert('hello');".to_string()));
        assert_eq!(deserialized.inject_css, Some("body { background: black; }".to_string()));
        assert!(deserialized.hide_on_close);
        assert!(deserialized.single_instance);
        assert!(deserialized.tray);
    }

    #[test]
    fn test_builtin_extensions_exist_and_protected() {
        let builtins = get_builtin_extensions();
        assert!(builtins.plugins.iter().any(|p| p.id == "builtin-wa-promo-cleaner"));
        assert!(builtins.themes.iter().any(|t| t.id == "builtin-wa-promo-css"));
        for p in &builtins.plugins {
            assert!(p.is_builtin);
        }
        for t in &builtins.themes {
            assert!(t.is_builtin);
        }
    }

    #[test]
    fn test_compile_app_extensions_order_and_accumulation() {
        let mut meta = AppMetadata {
            app_plugins: vec![
                AppExtensionRef {
                    id: "p1".to_string(),
                    name: "Plugin 1".to_string(),
                    category: "plugin".to_string(),
                    enabled: true,
                    custom_content: Some("console.log('first');".to_string()),
                },
                AppExtensionRef {
                    id: "p2".to_string(),
                    name: "Plugin 2".to_string(),
                    category: "plugin".to_string(),
                    enabled: false,
                    custom_content: Some("console.log('disabled');".to_string()),
                },
                AppExtensionRef {
                    id: "p3".to_string(),
                    name: "Plugin 3".to_string(),
                    category: "plugin".to_string(),
                    enabled: true,
                    custom_content: Some("console.log('third');".to_string()),
                },
            ],
            app_themes: vec![
                AppExtensionRef {
                    id: "t1".to_string(),
                    name: "Theme 1".to_string(),
                    category: "theme".to_string(),
                    enabled: true,
                    custom_content: Some("body { color: red; }".to_string()),
                },
            ],
            ..Default::default()
        };

        compile_app_extensions(&mut meta);
        assert_eq!(meta.inject_js, Some("console.log('first');\n\nconsole.log('third');".to_string()));
        assert_eq!(meta.inject_css, Some("body { color: red; }".to_string()));
    }

    #[test]
    fn test_delete_builtin_fails() {
        let res = delete_registry_item_cmd("builtin-wa-promo-cleaner".to_string());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Cannot delete built-in"));
    }

    #[test]
    fn test_starter_marketplace_catalog_validity() {
        let catalog = get_starter_marketplace_catalog();
        assert_eq!(catalog.total_items, 4);
        assert_eq!(catalog.plugins_count, 2);
        assert_eq!(catalog.themes_count, 2);
        assert!(catalog.items.iter().any(|i| i.id == "github-wide-diffs"));
        assert!(catalog.items.iter().any(|i| i.id == "youtube-ambient-control"));
        assert!(catalog.items.iter().any(|i| i.id == "oled-true-black"));
        assert!(catalog.items.iter().any(|i| i.id == "nord-frost"));
    }

    #[test]
    fn test_install_marketplace_item_fallback() {
        let res = install_marketplace_item("nord-frost");
        assert!(res.is_ok());
        let item = res.unwrap();
        assert_eq!(item.id, "nord-frost");
        assert_eq!(item.category, "theme");
        assert!(item.content.contains("--nord0"));

        let reg = load_registry();
        assert!(reg.themes.iter().any(|t| t.id == "nord-frost"));
    }
}


