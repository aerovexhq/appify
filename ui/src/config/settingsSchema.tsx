import {
  ShieldCheck,
  Cpu,
  Globe,
  Monitor,
  Check,
} from 'lucide-react';
import {
  SettingRow,
  Switch,
  Select,
  Range,
  TextInput,
  Button,
  type SettingsSectionGroup,
} from '@aerovex/strata-ui';
import type { AppifyGlobalSettings } from '../types';

export type SettingsUpdater = <K extends keyof AppifyGlobalSettings>(
  key: K,
  value: AppifyGlobalSettings[K]
) => void;

export interface SettingsSchemaHandlers {
  onSelfInstall?: () => void;
  selfInstalled?: boolean;
}

export function getAppifySettingsSchema(
  settings: AppifyGlobalSettings,
  updateSetting: SettingsUpdater,
  handlers: SettingsSchemaHandlers = {}
): SettingsSectionGroup[] {
  return [
    {
      label: 'Engine & Profiles',
      items: [
        {
          id: 'profiles',
          label: 'WebKit Profile Isolation',
          description: 'Data partition policies, storage quotas, and client identity',
          icon: <ShieldCheck className="w-4 h-4 text-accent" />,
          keywords: ['webkit', 'profile', 'isolation', 'cookie', 'partition', 'storage', 'cache'],
          content: (
            <div className="space-y-1">
              <SettingRow
                title="Per-App Profile Partitioning"
                subtitle="Isolate cookies, localStorage, IndexedDB, and credentials into unique SHA-256 directories"
              >
                <Switch
                  checked={settings.partitionProfiles}
                  onCheckedChange={(checked) => updateSetting('partitionProfiles', checked)}
                  label="Profile Partitioning"
                />
              </SettingRow>

              <SettingRow
                title="HTTP Disk Cache Quota"
                subtitle={`Maximum allocated cache storage per isolated profile (${settings.httpDiskCacheQuotaMb} MB)`}
              >
                <div className="w-48">
                  <Range
                    min={64}
                    max={2048}
                    step={64}
                    label="Disk Cache Quota"
                    value={settings.httpDiskCacheQuotaMb}
                    onValueChange={(val: number) => updateSetting('httpDiskCacheQuotaMb', val)}
                    format={(v) => `${v} MB`}
                  />
                </div>
              </SettingRow>

              <SettingRow
                title="Default Client User-Agent"
                subtitle="Custom User-Agent string reported by WebKit instances"
              >
                <div className="w-64">
                  <TextInput
                    label="User-Agent String"
                    hideLabel
                    value={settings.defaultUserAgent}
                    onChange={(val: string) => updateSetting('defaultUserAgent', val)}
                    placeholder="Leave blank for native default"
                  />
                </div>
              </SettingRow>
            </div>
          ),
        },
      ],
    },
    {
      label: 'Hardware & Rendering',
      items: [
        {
          id: 'hardware',
          label: 'GPU Acceleration',
          description: 'Hardware rasterization, DMABuf pipeline, and WebGL compositing',
          icon: <Cpu className="w-4 h-4 text-accent" />,
          keywords: ['gpu', 'hardware', 'acceleration', 'dmabuf', 'canvas', 'webgl', 'render'],
          content: (
            <div className="space-y-1">
              <SettingRow
                title="WebKitGTK DMABuf Renderer"
                subtitle="Zero-copy direct memory access buffer sharing for Linux GPU rendering"
              >
                <Switch
                  checked={settings.enableDmaBufRenderer}
                  onCheckedChange={(checked) => updateSetting('enableDmaBufRenderer', checked)}
                  label="DMABuf Renderer"
                />
              </SettingRow>

              <SettingRow
                title="Hardware Accelerated 2D Canvas"
                subtitle="Accelerate 2D vector drawing operations using GPU rasterization cores"
              >
                <Switch
                  checked={settings.enableHardware2dCanvas}
                  onCheckedChange={(checked) => updateSetting('enableHardware2dCanvas', checked)}
                  label="Hardware 2D Canvas"
                />
              </SettingRow>

              <SettingRow
                title="WebGL 2.0 Compositing"
                subtitle="Enable interactive 3D WebGL and WebGPU shader pipelines"
              >
                <Switch
                  checked={settings.enableWebGlCompositing}
                  onCheckedChange={(checked) => updateSetting('enableWebGlCompositing', checked)}
                  label="WebGL Compositing"
                />
              </SettingRow>
            </div>
          ),
        },
      ],
    },
    {
      label: 'Network & Privacy',
      items: [
        {
          id: 'network',
          label: 'Network & Proxy',
          description: 'Connection protocol routing, proxy tunnels, and telemetry blocking',
          icon: <Globe className="w-4 h-4 text-accent" />,
          keywords: ['network', 'proxy', 'socks5', 'http', 'privacy', 'tracker', 'adblock'],
          content: (
            <div className="space-y-1">
              <SettingRow
                title="Proxy Connection Protocol"
                subtitle="Route outbound application traffic through local or remote proxies"
              >
                <div className="w-44">
                  <Select
                    label="Proxy Protocol"
                    value={settings.proxyProtocol}
                    onValueChange={(val: string) => updateSetting('proxyProtocol', val as any)}
                    options={[
                      { value: 'direct', label: 'Direct Connection' },
                      { value: 'socks5', label: 'SOCKS5 Proxy' },
                      { value: 'http', label: 'HTTP Tunnel' },
                    ]}
                  />
                </div>
              </SettingRow>

              {settings.proxyProtocol !== 'direct' && (
                <SettingRow
                  title="Proxy Server Address"
                  subtitle="Host address and port (e.g. 127.0.0.1:9050 or proxy.corp:8080)"
                >
                  <div className="w-64">
                    <TextInput
                      label="Proxy Server Host"
                      hideLabel
                      value={settings.proxyHost}
                      onChange={(val: string) => updateSetting('proxyHost', val)}
                      placeholder="127.0.0.1:9050"
                    />
                  </div>
                </SettingRow>
              )}

              <SettingRow
                title="Strict Tracker & Telemetry Blocking"
                subtitle="Cosmetic and script filtering to block third-party analytics in wrapped web apps"
              >
                <Switch
                  checked={settings.strictTrackerBlocking}
                  onCheckedChange={(checked) => updateSetting('strictTrackerBlocking', checked)}
                  label="Strict Tracker Blocking"
                />
              </SettingRow>
            </div>
          ),
        },
      ],
    },
    {
      label: 'Desktop Integration',
      items: [
        {
          id: 'system',
          label: 'System & Daemon',
          description: 'Global CLI self-installation and session autostart',
          icon: <Monitor className="w-4 h-4 text-accent" />,
          keywords: ['system', 'daemon', 'install', 'binary', 'cli', 'autostart'],
          content: (
            <div className="space-y-1">
              <SettingRow
                title="Install CLI Binary to System PATH"
                subtitle="Symlink binary to /usr/local/bin/appify for terminal command-line access"
              >
                <Button
                  variant={handlers.selfInstalled ? 'primary' : 'secondary'}
                  onClick={handlers.onSelfInstall}
                  className="h-8 px-3 text-xs"
                >
                  {handlers.selfInstalled ? (
                    <span className="flex items-center gap-1.5">
                      <Check className="w-3.5 h-3.5" />
                      <span>Binary Installed</span>
                    </span>
                  ) : (
                    <span>Self-Install Binary</span>
                  )}
                </Button>
              </SettingRow>

              <SettingRow
                title="Session Tray Daemon"
                subtitle="Keep background manager alive in system tray when all application windows are closed"
              >
                <Switch
                  checked={settings.autoLaunchDesktopDaemon}
                  onCheckedChange={(checked) => updateSetting('autoLaunchDesktopDaemon', checked)}
                  label="Session Tray Daemon"
                />
              </SettingRow>
            </div>
          ),
        },
      ],
    },
  ];
}
