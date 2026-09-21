use clap::{Parser, Subcommand};
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
    command: Commands,
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
    },
    /// Uninstall an app by URL or alias
    Uninstall {
        url: String,
    },
    /// List all installed applications
    List,
    /// Install the appify binary itself to ~/.local/bin (or system PATH)
    SelfInstall,
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
}

#[derive(Debug, Clone)]
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

    // 3. Fallback for navigator.clipboard.read if missing
    try {
        if (navigator.clipboard && !navigator.clipboard.read) {
            navigator.clipboard.read = function() {
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

fn resolve_metadata(
    raw_input: &str,
    opts: RunOptions,
) -> Result<AppMetadata, String> {
    let aliases = get_popular_aliases();
    let lower_input = raw_input.trim().to_lowercase();

    let (mut raw_url, default_name, default_wm, alias_domains) =
        if let Some(info) = aliases.get(lower_input.as_str()) {
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
    })
}

static FALLBACK_ICON_BYTES: &[u8] = include_bytes!("default_icon.png");

fn convert_to_png(bytes: &[u8]) -> Option<Vec<u8>> {
    let dyn_img = image::load_from_memory(bytes).ok()?;
    let mut png_buf = Vec::new();
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)
        .ok()?;
    Some(png_buf)
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

    if let Some(body) = html_body {
        let doc = scraper::Html::parse_document(&body);
        let link_sel = scraper::Selector::parse(
            "link[rel*='icon'], link[rel*='apple-touch-icon'], meta[property='og:image']",
        )
        .unwrap();

            let mut apple_icons = Vec::new();
            let mut other_icons = Vec::new();

            for el in doc.select(&link_sel) {
                let tag_name = el.value().name();
                if tag_name == "link" {
                    if let Some(href) = el.value().attr("href") {
                        let rel = el.value().attr("rel").unwrap_or("").to_lowercase();
                        if rel.contains("apple-touch-icon") {
                            apple_icons.push(href.to_string());
                        } else {
                            other_icons.push(href.to_string());
                        }
                    }
                } else if tag_name == "meta" {
                    if let Some(content) = el.value().attr("content") {
                        other_icons.push(content.to_string());
                    }
                }
            }
            candidate_urls.extend(apple_icons);
            candidate_urls.extend(other_icons);
        }

    if let Ok(base) = Url::parse(url_str) {
        if let Ok(fav) = base.join("/favicon.ico") {
            candidate_urls.push(fav.to_string());
        }
    } else {
        candidate_urls.push(format!("{}/favicon.ico", url_str));
    }

    let mut saved = false;
    let base_url = Url::parse(url_str).ok();

    for candidate in candidate_urls {
        let full_url = if let Some(ref base) = base_url {
            base.join(&candidate).map(|u| u.to_string()).unwrap_or(candidate)
        } else {
            candidate
        };

        if let Ok(resp) = agent
            .get(&full_url)
            .set(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            )
            .call()
        {
            let mut bytes = Vec::new();
            if resp.into_reader().read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                if let Some(png_bytes) = convert_to_png(&bytes) {
                    if fs::write(icon_path, png_bytes).is_ok() {
                        saved = true;
                        break;
                    }
                }
            }
        }
    }

    if !saved {
        let _ = fs::write(icon_path, FALLBACK_ICON_BYTES);
    }
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
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("appify")
            .join("profiles")
            .join(hash)
            .join("instance.port")
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

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("appify")
        .join("profiles")
        .join(&meta.hash);

    fs::create_dir_all(&data_dir).ok();

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
                .disable_drag_drop_handler()
                .enable_clipboard_access()
                .initialization_script(BROWSER_INTEGRATION_SCRIPT);

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

                            let download_dir = dirs::download_dir().unwrap_or_else(|| {
                                dirs::home_dir()
                                    .map(|h| h.join("Downloads"))
                                    .unwrap_or_else(|| PathBuf::from("."))
                            });
                            *destination = download_dir.join(filename);
                            true
                        }
                        _ => true,
                    }
                })
                .build()?;

            #[cfg(target_os = "linux")]
            {
                use webkit2gtk::{PermissionRequestExt, SettingsExt, WebViewExt};
                let _ = window.with_webview(|platform_webview| {
                    let wv = platform_webview.inner();
                    if let Some(settings) = wv.settings() {
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
                });
            }

            let _ = window.set_title(&win_title);

            if let Some(zoom_val) = meta.zoom {
                let _ = window.set_zoom(zoom_val);
            }

            if meta.hide_on_close {
                let w_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w_clone.hide();
                    }
                });
            }

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
                    let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                    let menu = tauri::menu::Menu::with_items(app, &[&show_hide_item, &reload_item, &quit_item])?;

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

fn execute_self_install() {
    let persistent = ensure_persistent_binary();
    println!("[+] Appify is installed at: {}", persistent.display());
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

fn execute_install(meta: AppMetadata) {
    let home = dirs::home_dir().expect("Cannot resolve home directory");
    let bin_path = ensure_persistent_binary();

    let apps_dir = home.join(".local/share/applications");
    let icons_dir = home.join(".local/share/icons");
    fs::create_dir_all(&apps_dir).ok();
    fs::create_dir_all(&icons_dir).ok();

    let icon_path = icons_dir.join(format!("appify-{}.png", &meta.hash));

    if let Some(ref custom) = meta.custom_icon {
        let p = Path::new(custom);
        if p.exists() {
            if let Ok(bytes) = fs::read(p) {
                if let Some(png) = convert_to_png(&bytes) {
                    fs::write(&icon_path, png).ok();
                } else {
                    fs::copy(p, &icon_path).ok();
                }
            }
        } else if custom.starts_with("http://") || custom.starts_with("https://") {
            println!("[*] Downloading provided icon URL...");
            fetch_icon(custom, &icon_path);
        }
    } else {
        println!("[*] Fetching app icon from website...");
        fetch_icon(&meta.url, &icon_path);
    }

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
        "Disabled".to_string()
    };

    println!("[+] Successfully installed '{}'!", meta.name);
    println!("    URL:       {}", meta.url);
    println!("    WM_CLASS:  {}", meta.wm_class);
    println!("    Icon:      {}", icon_path.display());
    println!("    Desktop:   {}", desktop_path.display());
    println!("    Autostart: {}", autostart_msg);
}

fn execute_uninstall(raw_url: &str) {
    let meta = match resolve_metadata(raw_url, RunOptions::default()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[!] {}", e);
            return;
        }
    };

    let home = dirs::home_dir().unwrap();
    let desktop_path = home.join(format!(".local/share/applications/appify-{}.desktop", &meta.hash));
    let icon_path = home.join(format!(".local/share/icons/appify-{}.png", &meta.hash));
    let profile_dir = dirs::data_local_dir().unwrap().join("appify/profiles").join(&meta.hash);

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
    if remove_autostart(&meta.hash) {
        println!("[*] Removed autostart entry.");
        removed = true;
    }

    if removed {
        println!("[+] Successfully uninstalled '{}'", meta.name);
    } else {
        println!("[!] App was not found.");
    }
}

fn execute_list() {
    let home = dirs::home_dir().unwrap();
    let apps_dir = home.join(".local/share/applications");

    println!(
        "{:<22} | {:<18} | {:<15} | {}",
        "App Name", "WM_CLASS", "Autostart", "URL"
    );
    println!("{}", "-".repeat(85));

    if let Ok(entries) = fs::read_dir(apps_dir) {
        let mut apps = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with("appify-") && s.ends_with(".desktop"))
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut name = String::new();
                    let mut wm = String::new();
                    let mut url = String::new();
                    let mut hash = String::new();

                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix("Name=") {
                            name = val.to_string();
                        } else if let Some(val) = line.strip_prefix("StartupWMClass=") {
                            wm = val.to_string();
                        } else if let Some(val) = line.strip_prefix("X-Appify-URL=") {
                            url = val.to_string();
                        } else if let Some(val) = line.strip_prefix("X-Appify-Hash=") {
                            hash = val.to_string();
                        }
                    }

                    if !url.is_empty() {
                        let autostart_str = get_autostart_status(&hash);
                        apps.push((name, wm, autostart_str, url));
                    }
                }
            }
        }
        apps.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, wm, autostart, url) in apps {
            println!("{:<22} | {:<18} | {:<15} | {}", name, wm, autostart, url);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
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
        } => {
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
            };
            match resolve_metadata(&url, opts) {
                Ok(meta) => execute_install(meta),
                Err(e) => eprintln!("[!] Error: {}", e),
            }
        }
        Commands::Run {
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
        } => {
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
            };
            match resolve_metadata(&url, opts) {
                Ok(meta) => execute_run(meta),
                Err(e) => eprintln!("[!] Error: {}", e),
            }
        }
        Commands::Uninstall { url } => {
            execute_uninstall(&url);
        }
        Commands::List => {
            execute_list();
        }
        Commands::SelfInstall => {
            execute_self_install();
        }
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
    }
}

