export interface AppInfoDto {
  name: string;
  url: string;
  wm_class: string;
  hash: string;
  icon_path: string;
  icon_base64?: string | null;
  single_instance: boolean;
  hide_on_close: boolean;
  tray: boolean;
  start_hidden: boolean;
  autostart: boolean;
  autostart_hidden: boolean;
  zoom?: number | null;
  width?: number | null;
  height?: number | null;
  custom_allowed_domains: string[];
  has_custom_scripts: boolean;
  maximize?: boolean;
  user_agent?: string | null;
  is_running?: boolean;
}

export interface PresetInfoDto {
  id: string;
  name: string;
  description: string;
  category: string;
  url: string;
  wm_class: string;
  is_installed: boolean;
  installed_hash?: string | null;
}

export interface InstallRequestDto {
  url: string;
  name?: string | null;
  wm_class?: string | null;
  custom_icon?: string | null;
  custom_allowed_domains?: string[];
  hide_on_close?: boolean;
  single_instance?: boolean;
  tray?: boolean;
  start_hidden?: boolean;
  maximize?: boolean;
  zoom?: number | null;
  user_agent?: string | null;
  width?: number | null;
  height?: number | null;
  autostart?: boolean;
  autostart_hidden?: boolean;
  inject_js?: string | null;
  inject_css?: string | null;
}

export interface UpdateConfigDto {
  name?: string;
  wm_class?: string;
  custom_allowed_domains?: string[];
  hide_on_close?: boolean;
  single_instance?: boolean;
  tray?: boolean;
  start_hidden?: boolean;
  maximize?: boolean;
  zoom?: number | null;
  user_agent?: string | null;
  width?: number | null;
  height?: number | null;
  autostart?: boolean;
  autostart_hidden?: boolean;
  inject_js?: string | null;
  inject_css?: string | null;
  app_plugins?: AppExtensionRef[];
  app_themes?: AppExtensionRef[];
}

export interface AppExtensionRef {
  id: string;
  name: string;
  category: string; // 'plugin' | 'theme'
  enabled: boolean;
  custom_content?: string | null;
}

export interface ExtensionItem {
  id: string;
  name: string;
  description: string;
  category: string; // 'plugin' (JS) or 'theme' (CSS)
  content: string;
  author: string;
  version: string;
  is_builtin: boolean;
}

export interface ExtensionRegistry {
  plugins: ExtensionItem[];
  themes: ExtensionItem[];
}

export interface MarketplaceCatalogDto {
  version: string;
  cdn_base: string;
  generated_at: string;
  total_items: number;
  plugins_count: number;
  themes_count?: number;
  items?: ExtensionItem[];
}

export interface ScriptsDto {
  js: string;
  css: string;
  plugins: AppExtensionRef[];
  themes: AppExtensionRef[];
}

export interface SystemInfoDto {
  bin_path: string;
  profiles_dir: string;
  applications_dir: string;
  autostart_dir: string;
}

export type ActiveTab = 'apps' | 'presets' | 'wizard' | 'extensions' | 'marketplace' | 'diagnostics';

export interface AppifyGlobalSettings {
  partitionProfiles: boolean;
  httpDiskCacheQuotaMb: number;
  defaultUserAgent: string;
  enableDmaBufRenderer: boolean;
  enableHardware2dCanvas: boolean;
  enableWebGlCompositing: boolean;
  proxyProtocol: 'direct' | 'socks5' | 'http';
  proxyHost: string;
  strictTrackerBlocking: boolean;
  autoLaunchDesktopDaemon: boolean;
}
