import { create } from 'zustand';
import type {
  ActiveTab,
  AppInfoDto,
  PresetInfoDto,
  ExtensionRegistry,
  MarketplaceCatalogDto,
  SystemInfoDto,
  AppifyGlobalSettings,
  InstallRequestDto,
  UpdateConfigDto,
} from '../types';
import { bridge } from '../transport/bridge';

interface LogEntry {
  id: string;
  timestamp: string;
  level: 'info' | 'warn' | 'error';
  message: string;
}

interface AppifyState {
  // Navigation & UI
  activeTab: ActiveTab;
  searchQuery: string;
  presetCategory: string;
  settingsModalOpen: boolean;
  shortcutsModalOpen: boolean;
  editingApp: AppInfoDto | null;
  loading: boolean;
  error: string | null;

  // Data
  apps: AppInfoDto[];
  presets: PresetInfoDto[];
  registry: ExtensionRegistry;
  catalog: MarketplaceCatalogDto | null;
  systemInfo: SystemInfoDto | null;
  settings: AppifyGlobalSettings;
  logs: LogEntry[];

  // Setters
  setActiveTab: (tab: ActiveTab) => void;
  setSearchQuery: (query: string) => void;
  setPresetCategory: (cat: string) => void;
  setSettingsModalOpen: (open: boolean) => void;
  setShortcutsModalOpen: (open: boolean) => void;
  setEditingApp: (app: AppInfoDto | null) => void;
  updateSetting: <K extends keyof AppifyGlobalSettings>(key: K, value: AppifyGlobalSettings[K]) => void;
  addLog: (level: 'info' | 'warn' | 'error', message: string) => void;
  clearLogs: () => void;

  // Async actions
  loadAll: () => Promise<void>;
  fetchApps: () => Promise<void>;
  fetchPresets: () => Promise<void>;
  fetchRegistry: () => Promise<void>;
  fetchCatalog: () => Promise<void>;
  fetchSystemInfo: () => Promise<void>;
  launchApp: (hash: string) => Promise<void>;
  uninstallApp: (hash: string) => Promise<void>;
  clearCache: (hash: string) => Promise<void>;
  openFolder: (hash: string) => Promise<void>;
  installPreset: (id: string, options?: InstallRequestDto | null) => Promise<void>;
  installCustom: (req: InstallRequestDto) => Promise<AppInfoDto>;
  updateAppConfig: (hash: string, req: UpdateConfigDto, reinstallIcon?: boolean) => Promise<void>;
  installMarketplaceItem: (id: string) => Promise<void>;
  selfInstallBinary: () => Promise<string>;
}

export const useAppifyStore = create<AppifyState>((set, get) => ({
  activeTab: 'apps',
  searchQuery: '',
  presetCategory: 'all',
  settingsModalOpen: false,
  shortcutsModalOpen: false,
  editingApp: null,
  loading: false,
  error: null,

  apps: [],
  presets: [],
  registry: { plugins: [], themes: [] },
  catalog: null,
  systemInfo: null,
  settings: {
    partitionProfiles: true,
    httpDiskCacheQuotaMb: 512,
    defaultUserAgent: '',
    enableDmaBufRenderer: true,
    enableHardware2dCanvas: true,
    enableWebGlCompositing: true,
    proxyProtocol: 'direct',
    proxyHost: '',
    strictTrackerBlocking: true,
    autoLaunchDesktopDaemon: true,
  },
  logs: [
    {
      id: 'init_1',
      timestamp: new Date().toLocaleTimeString(),
      level: 'info',
      message: 'Appify desktop engine initialized with WebKitGTK profile isolation.',
    },
  ],

  setActiveTab: (activeTab) => set({ activeTab }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setPresetCategory: (presetCategory) => set({ presetCategory }),
  setSettingsModalOpen: (settingsModalOpen) => set({ settingsModalOpen }),
  setShortcutsModalOpen: (shortcutsModalOpen) => set({ shortcutsModalOpen }),
  setEditingApp: (editingApp) => set({ editingApp }),

  updateSetting: (key, value) =>
    set((state) => ({
      settings: { ...state.settings, [key]: value },
    })),

  addLog: (level, message) =>
    set((state) => ({
      logs: [
        ...state.logs.slice(-200),
        {
          id: `log_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
          timestamp: new Date().toLocaleTimeString(),
          level,
          message,
        },
      ],
    })),

  clearLogs: () => set({ logs: [] }),

  loadAll: async () => {
    set({ loading: true, error: null });
    try {
      await Promise.allSettled([
        get().fetchApps(),
        get().fetchPresets(),
        get().fetchRegistry(),
        get().fetchSystemInfo(),
      ]);
    } catch (err: any) {
      set({ error: err?.message || 'Failed to initialize Appify subsystems' });
    } finally {
      set({ loading: false });
    }
  },

  fetchApps: async () => {
    try {
      const apps = await bridge.getInstalledApps();
      set({ apps });
      get().addLog('info', `Discovered ${apps.length} isolated desktop applications.`);
    } catch (err: any) {
      get().addLog('error', `Failed to fetch installed applications: ${err?.message}`);
    }
  },

  fetchPresets: async () => {
    try {
      const presets = await bridge.getPresets();
      set({ presets });
    } catch (err: any) {
      get().addLog('error', `Failed to fetch curated presets: ${err?.message}`);
    }
  },

  fetchRegistry: async () => {
    try {
      const registry = await bridge.getRegistry();
      set({ registry });
    } catch (err: any) {
      get().addLog('error', `Failed to load extensions registry: ${err?.message}`);
    }
  },

  fetchCatalog: async () => {
    try {
      const catalog = await bridge.getMarketplaceCatalog();
      set({ catalog });
    } catch (err: any) {
      get().addLog('error', `Failed to load marketplace catalog: ${err?.message}`);
    }
  },

  fetchSystemInfo: async () => {
    try {
      const systemInfo = await bridge.getSystemInfo();
      set({ systemInfo });
    } catch (err: any) {
      get().addLog('warn', `Could not inspect system host info: ${err?.message}`);
    }
  },

  launchApp: async (hash: string) => {
    try {
      const target = get().apps.find((a) => a.hash === hash);
      get().addLog('info', `Spawning isolated process for "${target?.name || hash}"...`);
      await bridge.launchApp(hash);
      get().addLog('info', `Application "${target?.name || hash}" launched successfully.`);
    } catch (err: any) {
      get().addLog('error', `Failed to launch application: ${err?.message}`);
    }
  },

  uninstallApp: async (hash: string) => {
    try {
      const target = get().apps.find((a) => a.hash === hash);
      get().addLog('warn', `Uninstalling application "${target?.name || hash}" and purging profile...`);
      await bridge.uninstallApp(hash);
      await get().fetchApps();
      await get().fetchPresets();
      get().addLog('info', `Application "${target?.name || hash}" uninstalled.`);
    } catch (err: any) {
      get().addLog('error', `Failed to uninstall application: ${err?.message}`);
    }
  },

  clearCache: async (hash: string) => {
    try {
      await bridge.clearAppCache(hash);
      get().addLog('info', `Cleared cache for app profile (${hash}).`);
    } catch (err: any) {
      get().addLog('error', `Failed to clear cache: ${err?.message}`);
    }
  },

  openFolder: async (hash: string) => {
    try {
      await bridge.openAppFolder(hash);
      get().addLog('info', `Opened profile directory in desktop file manager.`);
    } catch (err: any) {
      get().addLog('error', `Failed to open profile folder: ${err?.message}`);
    }
  },

  installPreset: async (id: string, options = null) => {
    try {
      get().addLog('info', `Installing preset template "${id}"...`);
      await bridge.installPreset(id, options);
      await get().fetchApps();
      await get().fetchPresets();
      get().addLog('info', `Preset "${id}" installed successfully.`);
    } catch (err: any) {
      get().addLog('error', `Failed to install preset: ${err?.message}`);
      throw err;
    }
  },

  installCustom: async (req: InstallRequestDto) => {
    try {
      get().addLog('info', `Assembling native application for ${req.url}...`);
      const installed = await bridge.installCustom(req);
      await get().fetchApps();
      await get().fetchPresets();
      get().addLog('info', `Created desktop application "${installed.name}".`);
      return installed;
    } catch (err: any) {
      get().addLog('error', `Failed to assemble application: ${err?.message}`);
      throw err;
    }
  },

  updateAppConfig: async (hash: string, req: UpdateConfigDto, reinstallIcon = false) => {
    try {
      get().addLog('info', `Updating configuration for ${hash}...`);
      await bridge.updateAppConfig(hash, req, reinstallIcon);
      await get().fetchApps();
      get().addLog('info', `Configuration saved successfully.`);
    } catch (err: any) {
      get().addLog('error', `Failed to update configuration: ${err?.message}`);
      throw err;
    }
  },

  installMarketplaceItem: async (id: string) => {
    try {
      get().addLog('info', `Installing extension "${id}" from catalog...`);
      await bridge.installMarketplaceItem(id);
      await get().fetchRegistry();
      get().addLog('info', `Extension "${id}" installed.`);
    } catch (err: any) {
      get().addLog('error', `Failed to install marketplace extension: ${err?.message}`);
      throw err;
    }
  },

  selfInstallBinary: async () => {
    try {
      get().addLog('info', 'Invoking system binary self-installation...');
      const path = await bridge.selfInstallBinary();
      get().addLog('info', `Binary installed to ${path}.`);
      await get().fetchSystemInfo();
      return path;
    } catch (err: any) {
      get().addLog('error', `Self-install failed: ${err?.message}`);
      throw err;
    }
  },
}));
