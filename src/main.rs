use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
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

const NOTIFICATION_POLYFILL: &str = r#"
(function() {
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
    custom_name: Option<String>,
    custom_wm_class: Option<String>,
    custom_icon: Option<String>,
    cli_allowed_domains: Vec<String>,
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

    let final_name = custom_name.or(default_name).unwrap_or_else(|| netloc.clone());
    let final_wm = custom_wm_class
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
    for d in &cli_allowed_domains {
        allowed_set.insert(d.trim().to_lowercase());
    }

    let allowed_domains: Vec<String> = allowed_set.into_iter().collect();

    Ok(AppMetadata {
        url: raw_url,
        base_domain,
        name: final_name,
        wm_class: final_wm,
        custom_icon,
        hash,
        allowed_domains,
        custom_allowed_domains: cli_allowed_domains,
    })
}

fn fetch_icon(url_str: &str, icon_path: &Path) {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    let mut icon_url: Option<String> = None;

    if let Ok(resp) = agent.get(url_str).call() {
        if let Ok(body) = resp.into_string() {
            let doc = scraper::Html::parse_document(&body);
            let link_sel = scraper::Selector::parse("link[rel*='icon']").unwrap();

            for el in doc.select(&link_sel) {
                if let Some(href) = el.value().attr("href") {
                    let rel = el.value().attr("rel").unwrap_or("").to_lowercase();
                    if rel.contains("apple-touch-icon") {
                        icon_url = Some(href.to_string());
                        break;
                    } else if icon_url.is_none() {
                        icon_url = Some(href.to_string());
                    }
                }
            }
        }
    }

    let resolved_icon = if let Some(href) = icon_url {
        if let Ok(base) = Url::parse(url_str) {
            base.join(&href).map(|u| u.to_string()).unwrap_or_else(|_| format!("{}/favicon.ico", url_str))
        } else {
            format!("{}/favicon.ico", url_str)
        }
    } else {
        format!("{}/favicon.ico", url_str)
    };

    if let Ok(resp) = agent.get(&resolved_icon).call() {
        let mut bytes = Vec::new();
        if resp.into_reader().read_to_end(&mut bytes).is_ok() {
            let _ = fs::write(icon_path, bytes);
        }
    }
}

// Zero-dependency C FFI call for Linux process name
#[cfg(target_os = "linux")]
fn set_linux_app_id(app_id: &str) {
    use std::ffi::CString;
    extern "C" {
        fn prctl(option: i32, arg2: *const i8, arg3: u64, arg4: u64, arg5: u64) -> i32;
    }
    const PR_SET_NAME: i32 = 15;

    if let Ok(c_str) = CString::new(app_id) {
        unsafe {
            prctl(PR_SET_NAME, c_str.as_ptr(), 0, 0, 0);
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
        set_linux_app_id(&meta.wm_class);
    }

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("appify")
        .join("profiles")
        .join(&meta.hash);

    fs::create_dir_all(&data_dir).ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let handle_nav = app.handle().clone();
            let handle_new_win = app.handle().clone();

            let base_domain_for_nav = base_domain.clone();
            let allowed_domains_for_nav = allowed_domains.clone();

            let base_domain_for_new_win = base_domain.clone();
            let allowed_domains_for_new_win = allowed_domains.clone();

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(target_url),
            )
                .title(&win_title)
                .inner_size(1100.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .center()
                .data_directory(data_dir)
                .initialization_script(NOTIFICATION_POLYFILL);

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

            builder
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
                    let is_unread = title.starts_with('(') || title.contains("Unread") || title.contains("•");
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
            fs::copy(p, &icon_path).ok();
        } else if custom.starts_with("http://") || custom.starts_with("https://") {
            println!("[*] Downloading provided icon URL...");
            fetch_icon(custom, &icon_path);
        }
    } else {
        println!("[*] Fetching app icon from website...");
        fetch_icon(&meta.url, &icon_path);
    }

    let allow_flags = if !meta.custom_allowed_domains.is_empty() {
        let flags: Vec<String> = meta
            .custom_allowed_domains
            .iter()
            .map(|d| format!("--allow-domain \"{}\"", d))
            .collect();
        format!(" {}", flags.join(" "))
    } else {
        String::new()
    };

    let desktop_path = apps_dir.join(format!("appify-{}.desktop", &meta.hash));
    let desktop_content = format!(
        "[Desktop Entry]\n\
        Version=1.0\n\
        Type=Application\n\
        Name={name}\n\
        Exec={bin} run \"{url}\" \"{name}\" --wm-class \"{wm_class}\" --icon \"{icon}\"{allow_flags}\n\
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
        allow_flags = allow_flags,
        hash = meta.hash
    );

    let mut file = File::create(&desktop_path).expect("Failed to write desktop file");
    file.write_all(desktop_content.as_bytes()).unwrap();

    println!("[+] Successfully installed '{}'!", meta.name);
    println!("    URL:      {}", meta.url);
    println!("    WM_CLASS: {}", meta.wm_class);
    println!("    Icon:     {}", icon_path.display());
    println!("    Desktop:  {}", desktop_path.display());
}

fn execute_uninstall(raw_url: &str) {
    let meta = match resolve_metadata(raw_url, None, None, None, vec![]) {
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

    if removed {
        println!("[+] Successfully uninstalled '{}'", meta.name);
    } else {
        println!("[!] App was not found.");
    }
}

fn execute_list() {
    let home = dirs::home_dir().unwrap();
    let apps_dir = home.join(".local/share/applications");

    println!("{:<25} | {:<20} | {}", "App Name", "WM_CLASS", "URL");
    println!("{}", "-".repeat(75));

    if let Ok(entries) = fs::read_dir(apps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with("appify-") && s.ends_with(".desktop")) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut name = String::new();
                    let mut wm = String::new();
                    let mut url = String::new();

                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix("Name=") { name = val.to_string(); }
                        else if let Some(val) = line.strip_prefix("StartupWMClass=") { wm = val.to_string(); }
                        else if let Some(val) = line.strip_prefix("X-Appify-URL=") { url = val.to_string(); }
                    }

                    if !url.is_empty() {
                        println!("{:<25} | {:<20} | {}", name, wm, url);
                    }
                }
            }
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
        } => match resolve_metadata(&url, name, wm_class, icon, allow_domain) {
            Ok(meta) => execute_install(meta),
            Err(e) => eprintln!("[!] Error: {}", e),
        },
        Commands::Run {
            url,
            name,
            wm_class,
            icon,
            allow_domain,
        } => match resolve_metadata(&url, name, wm_class, icon, allow_domain) {
            Ok(meta) => execute_run(meta),
            Err(e) => eprintln!("[!] Error: {}", e),
        },
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
        let meta = resolve_metadata("whatsapp", None, None, None, vec![]).unwrap();
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
        let meta = resolve_metadata("  DISCORD  ", None, None, None, vec![]).unwrap();
        assert_eq!(meta.url, "https://discord.com/app");
        assert_eq!(meta.name, "Discord");
        assert_eq!(meta.wm_class, "discord-app");
        assert!(meta.allowed_domains.contains(&"discord.gg".to_string()));
    }

    #[test]
    fn test_resolve_metadata_raw_url_no_scheme() {
        let meta = resolve_metadata("github.com", None, None, None, vec![]).unwrap();
        assert_eq!(meta.url, "https://github.com");
        assert_eq!(meta.name, "github.com");
        assert_eq!(meta.base_domain, "github.com");
        assert!(meta.wm_class.starts_with("appify-"));
    }

    #[test]
    fn test_resolve_metadata_trailing_slash_stripped() {
        let meta = resolve_metadata("https://example.org/", None, None, None, vec![]).unwrap();
        assert_eq!(meta.url, "https://example.org");
    }

    #[test]
    fn test_resolve_metadata_custom_overrides() {
        let meta = resolve_metadata(
            "https://linear.app",
            Some("Linear Work".to_string()),
            Some("custom-linear".to_string()),
            Some("/path/to/icon.png".to_string()),
            vec!["linear.com".to_string()],
        )
        .unwrap();

        assert_eq!(meta.name, "Linear Work");
        assert_eq!(meta.wm_class, "custom-linear");
        assert_eq!(meta.custom_icon, Some("/path/to/icon.png".to_string()));
        assert!(meta.allowed_domains.contains(&"linear.com".to_string()));
        assert_eq!(meta.custom_allowed_domains, vec!["linear.com".to_string()]);
    }

    #[test]
    fn test_resolve_metadata_localhost() {
        let meta = resolve_metadata("http://localhost:3000", None, None, None, vec![]).unwrap();
        assert_eq!(meta.url, "http://localhost:3000");
        assert_eq!(meta.base_domain, "localhost");
    }

    #[test]
    fn test_resolve_metadata_invalid_domain() {
        let res = resolve_metadata("notadomain", None, None, None, vec![]);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not appear to be a valid domain"));
    }

    #[test]
    fn test_hash_consistency() {
        let meta1 = resolve_metadata("https://github.com", None, None, None, vec![]).unwrap();
        let meta2 = resolve_metadata("github.com", None, None, None, vec![]).unwrap();
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

        // Internal WhatsApp flows / cache URLs that previously leaked to default browser
        let url_flows = Url::parse("https://flows.whatsapp.net/flows/cache_management/").unwrap();
        assert!(is_internal_navigation(&url_flows, base, &allowed));

        let url_pdf = Url::parse("https://webtp.whatsapp.net/pdf-viewer/?locale=en_GB").unwrap();
        assert!(is_internal_navigation(&url_pdf, base, &allowed));

        let url_main = Url::parse("https://web.whatsapp.com").unwrap();
        assert!(is_internal_navigation(&url_main, base, &allowed));

        // OAuth SSO
        let url_sso = Url::parse("https://accounts.google.com/o/oauth2/v2/auth?client_id=123").unwrap();
        assert!(is_internal_navigation(&url_sso, base, &allowed));

        // External untrusted website
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
}
