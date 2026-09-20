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
    },
    /// Uninstall an app by URL or alias
    Uninstall {
        url: String,
    },
    /// List all installed applications
    List,
}

#[derive(Debug, Clone)]
struct AppMetadata {
    url: String,
    base_domain: String,
    name: String,
    wm_class: String,
    custom_icon: Option<String>,
    hash: String,
}

fn get_popular_aliases() -> HashMap<&'static str, (&'static str, &'static str, &'static str)> {
    let mut map = HashMap::new();
    map.insert("whatsapp", ("https://web.whatsapp.com", "WhatsApp", "whatsapp-desktop"));
    map.insert("discord", ("https://discord.com/app", "Discord", "discord-app"));
    map.insert("telegram", ("https://web.telegram.org", "Telegram", "telegram-web"));
    map.insert("spotify", ("https://open.spotify.com", "Spotify", "spotify-web"));
    map.insert("netflix", ("https://www.netflix.com", "Netflix", "netflix-app"));
    map.insert("youtube", ("https://youtube.com", "YouTube", "youtube-app"));
    map.insert("twitter", ("https://x.com", "X", "twitter-x"));
    map.insert("x", ("https://x.com", "X", "twitter-x"));
    map.insert("reddit", ("https://reddit.com", "Reddit", "reddit-app"));
    map.insert("chatgpt", ("https://chatgpt.com", "ChatGPT", "chatgpt-app"));
    map.insert("notion", ("https://notion.so", "Notion", "notion-app"));
    map.insert("figma", ("https://figma.com", "Figma", "figma-app"));
    map.insert("gmail", ("https://mail.google.com", "Gmail", "gmail-app"));
    map
}

fn resolve_metadata(
    raw_input: &str,
    custom_name: Option<String>,
    custom_wm_class: Option<String>,
    custom_icon: Option<String>,
) -> Result<AppMetadata, String> {
    let aliases = get_popular_aliases();
    let lower_input = raw_input.trim().to_lowercase();

    let (mut raw_url, default_name, default_wm) = if let Some(&(u, n, wm)) = aliases.get(lower_input.as_str()) {
        (u.to_string(), Some(n.to_string()), Some(wm.to_string()))
    } else {
        (raw_input.trim().to_string(), None, None)
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

    Ok(AppMetadata {
        url: raw_url,
        base_domain,
        name: final_name,
        wm_class: final_wm,
        custom_icon,
        hash,
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
            let handle = app.handle().clone();

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(target_url),
            )
                .title(&win_title)
                .inner_size(1100.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .center()
                .data_directory(data_dir);

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
                    let host = nav_url.host_str().unwrap_or("");
                    let scheme = nav_url.scheme();

                    if host.contains(&base_domain) || scheme == "blob" || scheme == "data" {
                        true
                    } else {
                        let _ = handle.opener().open_url(nav_url.as_str(), None::<&str>);
                        false
                    }
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Appify instance");
}

fn execute_install(meta: AppMetadata) {
    let home = dirs::home_dir().expect("Cannot resolve home directory");
    let bin_path = std::env::current_exe().expect("Failed to find current executable path");

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

    let desktop_path = apps_dir.join(format!("appify-{}.desktop", &meta.hash));
    let desktop_content = format!(
        "[Desktop Entry]\n\
        Version=1.0\n\
        Type=Application\n\
        Name={name}\n\
        Exec={bin} run \"{url}\" \"{name}\" --wm-class \"{wm_class}\" --icon \"{icon}\"\n\
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
    let meta = match resolve_metadata(raw_url, None, None, None) {
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
        Commands::Install { url, name, wm_class, icon } => {
            match resolve_metadata(&url, name, wm_class, icon) {
                Ok(meta) => execute_install(meta),
                Err(e) => eprintln!("[!] Error: {}", e),
            }
        }
        Commands::Run { url, name, wm_class, icon } => {
            match resolve_metadata(&url, name, wm_class, icon) {
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

        for (alias, (url, name, wm)) in &aliases {
            assert!(!alias.is_empty());
            assert!(url.starts_with("https://"));
            assert!(!name.is_empty());
            assert!(!wm.is_empty());
        }
    }

    #[test]
    fn test_resolve_metadata_alias() {
        let meta = resolve_metadata("whatsapp", None, None, None).unwrap();
        assert_eq!(meta.url, "https://web.whatsapp.com");
        assert_eq!(meta.name, "WhatsApp");
        assert_eq!(meta.wm_class, "whatsapp-desktop");
        assert_eq!(meta.base_domain, "whatsapp.com");
        assert_eq!(meta.hash.len(), 64);
    }

    #[test]
    fn test_resolve_metadata_case_insensitive_alias() {
        let meta = resolve_metadata("  DISCORD  ", None, None, None).unwrap();
        assert_eq!(meta.url, "https://discord.com/app");
        assert_eq!(meta.name, "Discord");
        assert_eq!(meta.wm_class, "discord-app");
    }

    #[test]
    fn test_resolve_metadata_raw_url_no_scheme() {
        let meta = resolve_metadata("github.com", None, None, None).unwrap();
        assert_eq!(meta.url, "https://github.com");
        assert_eq!(meta.name, "github.com");
        assert_eq!(meta.base_domain, "github.com");
        assert!(meta.wm_class.starts_with("appify-"));
    }

    #[test]
    fn test_resolve_metadata_trailing_slash_stripped() {
        let meta = resolve_metadata("https://example.org/", None, None, None).unwrap();
        assert_eq!(meta.url, "https://example.org");
    }

    #[test]
    fn test_resolve_metadata_custom_overrides() {
        let meta = resolve_metadata(
            "https://linear.app",
            Some("Linear Work".to_string()),
            Some("custom-linear".to_string()),
            Some("/path/to/icon.png".to_string()),
        )
        .unwrap();

        assert_eq!(meta.name, "Linear Work");
        assert_eq!(meta.wm_class, "custom-linear");
        assert_eq!(meta.custom_icon, Some("/path/to/icon.png".to_string()));
    }

    #[test]
    fn test_resolve_metadata_localhost() {
        let meta = resolve_metadata("http://localhost:3000", None, None, None).unwrap();
        assert_eq!(meta.url, "http://localhost:3000");
        assert_eq!(meta.base_domain, "localhost");
    }

    #[test]
    fn test_resolve_metadata_invalid_domain() {
        let res = resolve_metadata("notadomain", None, None, None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not appear to be a valid domain"));
    }

    #[test]
    fn test_hash_consistency() {
        let meta1 = resolve_metadata("https://github.com", None, None, None).unwrap();
        let meta2 = resolve_metadata("github.com", None, None, None).unwrap();
        assert_eq!(meta1.hash, meta2.hash);
    }
}
