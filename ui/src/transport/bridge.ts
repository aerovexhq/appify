import { bridge as strataBridge } from '@aerovex/strata-bridge';
import type {
  AppInfoDto,
  PresetInfoDto,
  InstallRequestDto,
  UpdateConfigDto,
  ScriptsDto,
  SystemInfoDto,
  ExtensionItem,
  ExtensionRegistry,
  MarketplaceCatalogDto,
} from '../types';

export class AppifyBridge {
  static get platform() {
    return strataBridge.platform;
  }

  static get isNative() {
    return strataBridge.isNative;
  }

  static async invoke<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await strataBridge.invoke<T>(command, args);
    } catch (err: unknown) {
      // Fallback to window.__TAURI__.core.invoke if available
      if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
        return (window as any).__TAURI__.core.invoke(command, args);
      }
      throw err;
    }
  }

  // App management
  static async getInstalledApps(): Promise<AppInfoDto[]> {
    return this.invoke<AppInfoDto[]>('get_installed_apps');
  }

  static async launchApp(hash: string): Promise<void> {
    return this.invoke<void>('launch_app_cmd', { hash });
  }

  static async uninstallApp(hash: string): Promise<void> {
    return this.invoke<void>('uninstall_app_cmd', { hash });
  }

  static async updateAppConfig(hash: string, req: UpdateConfigDto, reinstallIcon?: boolean): Promise<AppInfoDto> {
    return this.invoke<AppInfoDto>('update_app_config_cmd', { hash, req, reinstall_icon: reinstallIcon });
  }

  static async clearAppCache(hash: string): Promise<void> {
    return this.invoke<void>('clear_app_cache_cmd', { hash });
  }

  static async openAppFolder(hash: string): Promise<void> {
    return this.invoke<void>('open_app_folder_cmd', { hash });
  }

  // Presets
  static async getPresets(): Promise<PresetInfoDto[]> {
    return this.invoke<PresetInfoDto[]>('get_presets');
  }

  static async installPreset(id: string, options?: InstallRequestDto | null): Promise<AppInfoDto> {
    return this.invoke<AppInfoDto>('install_preset_cmd', { id, options: options || null });
  }

  // Custom App Creation
  static async installCustom(req: InstallRequestDto): Promise<AppInfoDto> {
    return this.invoke<AppInfoDto>('install_custom_cmd', { req });
  }

  // Scripts & UserStyles
  static async getAppScripts(hash: string): Promise<ScriptsDto> {
    return this.invoke<ScriptsDto>('get_app_scripts_cmd', { hash });
  }

  static async saveAppScripts(
    hash: string,
    js: string,
    css: string,
    plugins: any[],
    themes: any[]
  ): Promise<void> {
    return this.invoke<void>('save_app_scripts_cmd', { hash, js, css, plugins, themes });
  }

  // Extension Registry
  static async getRegistry(): Promise<ExtensionRegistry> {
    return this.invoke<ExtensionRegistry>('get_registry_cmd');
  }

  static async saveRegistryItem(item: ExtensionItem): Promise<void> {
    return this.invoke<void>('save_registry_item_cmd', { item });
  }

  static async deleteRegistryItem(id: string): Promise<void> {
    return this.invoke<void>('delete_registry_item_cmd', { id });
  }

  // Marketplace
  static async getMarketplaceCatalog(): Promise<MarketplaceCatalogDto> {
    return this.invoke<MarketplaceCatalogDto>('get_marketplace_catalog_cmd');
  }

  static async installMarketplaceItem(id: string): Promise<ExtensionItem> {
    return this.invoke<ExtensionItem>('install_marketplace_item_cmd', { id });
  }

  // System Diagnostics
  static async getSystemInfo(): Promise<SystemInfoDto> {
    return this.invoke<SystemInfoDto>('get_system_info_cmd');
  }

  static async selfInstallBinary(): Promise<string> {
    return this.invoke<string>('self_install_binary_cmd');
  }
}

// Register offline web development mocks if running outside native desktop
if (!strataBridge.isNative) {
  strataBridge.registerMock('get_installed_apps', () => [
    {
      name: 'GitHub',
      url: 'https://github.com',
      wm_class: 'appify-github',
      hash: 'mock_gh_123',
      icon_path: '',
      icon_base64: null,
      single_instance: true,
      hide_on_close: false,
      tray: true,
      start_hidden: false,
      autostart: false,
      autostart_hidden: false,
      zoom: 1.0,
      width: 1280,
      height: 800,
      custom_allowed_domains: ['github.com', 'githubassets.com'],
      has_custom_scripts: true,
      maximize: false,
      user_agent: null,
    },
    {
      name: 'Linear',
      url: 'https://linear.app',
      wm_class: 'appify-linear',
      hash: 'mock_linear_456',
      icon_path: '',
      icon_base64: null,
      single_instance: true,
      hide_on_close: true,
      tray: true,
      start_hidden: false,
      autostart: true,
      autostart_hidden: false,
      zoom: 1.0,
      width: 1400,
      height: 900,
      custom_allowed_domains: ['linear.app'],
      has_custom_scripts: false,
      maximize: false,
      user_agent: null,
    },
  ]);

  strataBridge.registerMock('get_presets', () => [
    {
      id: 'github',
      name: 'GitHub',
      description: 'Host and review code, manage projects, and build software alongside developers.',
      category: 'Developer',
      url: 'https://github.com',
      wm_class: 'appify-github',
      is_installed: true,
      installed_hash: 'mock_gh_123',
    },
    {
      id: 'linear',
      name: 'Linear',
      description: 'The issue tracking tool you will enjoy using. Built for high-performance software teams.',
      category: 'Productivity',
      url: 'https://linear.app',
      wm_class: 'appify-linear',
      is_installed: true,
      installed_hash: 'mock_linear_456',
    },
    {
      id: 'discord',
      name: 'Discord',
      description: 'Chat, voice, and video messaging platform for communities and engineering teams.',
      category: 'Communication',
      url: 'https://discord.com/app',
      wm_class: 'appify-discord',
      is_installed: false,
      installed_hash: null,
    },
    {
      id: 'spotify',
      name: 'Spotify',
      description: 'Digital music service that gives you access to millions of songs and podcasts.',
      category: 'Media',
      url: 'https://open.spotify.com',
      wm_class: 'appify-spotify',
      is_installed: false,
      installed_hash: null,
    },
    {
      id: 'notion',
      name: 'Notion',
      description: 'Connected workspace where better, faster work happens.',
      category: 'Productivity',
      url: 'https://www.notion.so',
      wm_class: 'appify-notion',
      is_installed: false,
      installed_hash: null,
    },
  ]);

  strataBridge.registerMock('get_system_info_cmd', () => ({
    bin_path: '/usr/local/bin/appify',
    profiles_dir: '~/.local/share/appify/profiles',
    applications_dir: '~/.local/share/applications',
    autostart_dir: '~/.config/autostart',
  }));

  strataBridge.registerMock('get_registry_cmd', () => ({
    plugins: [
      {
        id: 'dark-reader',
        name: 'Dark Reader Core',
        description: 'Inverts high-brightness DOM elements on pages lacking native dark theme.',
        category: 'plugin',
        content: '// Inverted dark mode polyfill\nconsole.log("[Appify Plugin] Dark Reader loaded");',
        author: 'Aerovex Systems',
        version: '1.2.0',
        is_builtin: true,
      },
    ],
    themes: [
      {
        id: 'obsidian-contrast',
        name: 'Obsidian High Contrast',
        description: 'Hairline borders and deep charcoal backgrounds for enterprise workflows.',
        category: 'theme',
        content: 'body { background-color: #08090a !important; color: #f3f4f6 !important; }',
        author: 'Aerovex Systems',
        version: '1.0.0',
        is_builtin: true,
      },
    ],
  }));

  strataBridge.registerMock('get_marketplace_catalog_cmd', () => ({
    version: '1.0.0',
    cdn_base: 'https://cdn.aerovex.net/appify/marketplace',
    generated_at: new Date().toISOString(),
    total_items: 2,
    plugins_count: 1,
    themes_count: 1,
    items: [
      {
        id: 'adblock-lite',
        name: 'AdBlock Lite',
        description: 'Zero-overhead cosmetic and banner filtering for clean reading.',
        category: 'plugin',
        content: '// AdBlock Lite script',
        author: 'Aerovex Systems',
        version: '1.0.4',
        is_builtin: false,
      },
      {
        id: 'cyber-cyan',
        name: 'Cyber Cyan UI',
        description: 'Neon electric accents for web clients.',
        category: 'theme',
        content: 'a { color: #00e5ff !important; }',
        author: 'Aerovex Systems',
        version: '1.1.0',
        is_builtin: false,
      },
    ],
  }));

  strataBridge.registerMock('launch_app_cmd', () => undefined);
  strataBridge.registerMock('uninstall_app_cmd', () => undefined);
  strataBridge.registerMock('clear_app_cache_cmd', () => undefined);
  strataBridge.registerMock('open_app_folder_cmd', () => undefined);
  strataBridge.registerMock('self_install_binary_cmd', () => '/usr/local/bin/appify');
}

export const bridge = AppifyBridge;
