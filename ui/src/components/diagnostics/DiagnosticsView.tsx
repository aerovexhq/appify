import { useState } from 'react';
import { Terminal, Shield, HardDrive, Check, Download } from 'lucide-react';
import { Button, LogConsole } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';

export function DiagnosticsView() {
  const { systemInfo, logs, clearLogs, selfInstallBinary, apps, clearCache } = useAppifyStore();
  const [installing, setInstalling] = useState(false);
  const [installedPath, setInstalledPath] = useState<string | null>(null);
  const [clearingAll, setClearingAll] = useState(false);

  const handleSelfInstall = async () => {
    setInstalling(true);
    try {
      const path = await selfInstallBinary();
      setInstalledPath(path);
    } finally {
      setInstalling(false);
    }
  };

  const handleClearAllCaches = async () => {
    setClearingAll(true);
    try {
      for (const app of apps) {
        await clearCache(app.hash);
      }
    } finally {
      setClearingAll(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col min-h-0 px-6 py-4 space-y-4">
      <div>
        <h1 className="text-lg font-semibold text-fg tracking-tight">Diagnostics & Runtime Environment</h1>
        <p className="text-xs text-muted">
          System paths, WebKit profile data directories, execution event telemetry, and cache maintenance.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div className="p-4 rounded-xl bg-surface border border-border space-y-3">
          <div className="flex items-center gap-2">
            <HardDrive className="w-4 h-4 text-accent" />
            <span className="text-xs font-semibold text-fg">Host File System Topology</span>
          </div>

          <div className="space-y-2 text-xs font-mono">
            <div>
              <span className="text-muted block text-[10px]">Profiles Root</span>
              <span className="text-fg truncate block bg-raised px-2 py-1 rounded border border-border">
                {systemInfo?.profiles_dir || '~/.local/share/appify/profiles'}
              </span>
            </div>
            <div>
              <span className="text-muted block text-[10px]">Desktop Applications (.desktop)</span>
              <span className="text-fg truncate block bg-raised px-2 py-1 rounded border border-border">
                {systemInfo?.applications_dir || '~/.local/share/applications'}
              </span>
            </div>
            <div>
              <span className="text-muted block text-[10px]">Session Autostart Entries</span>
              <span className="text-fg truncate block bg-raised px-2 py-1 rounded border border-border">
                {systemInfo?.autostart_dir || '~/.config/autostart'}
              </span>
            </div>
          </div>
        </div>

        <div className="p-4 rounded-xl bg-surface border border-border flex flex-col justify-between">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-accent" />
              <span className="text-xs font-semibold text-fg">System Binary & Maintenance</span>
            </div>
            <p className="text-xs text-muted leading-relaxed">
              Install the Appify CLI binary to <code className="text-fg font-mono">/usr/local/bin/appify</code> to invoke headless application packaging and WebKit profile management from your terminal.
            </p>
          </div>

          <div className="flex flex-wrap gap-2 pt-3 border-t border-border mt-3">
            <Button
              variant={installedPath ? 'secondary' : 'primary'}
              onClick={handleSelfInstall}
              disabled={installing}
              className="h-8 px-3 text-xs gap-1.5 justify-center"
            >
              {installedPath ? (
                <>
                  <Check className="w-3.5 h-3.5 text-emerald-400" />
                  <span>Installed to {installedPath}</span>
                </>
              ) : (
                <>
                  <Download className="w-3.5 h-3.5" />
                  <span>{installing ? 'Installing...' : 'Self-Install CLI to PATH'}</span>
                </>
              )}
            </Button>

            <Button
              variant="secondary"
              onClick={handleClearAllCaches}
              disabled={clearingAll || apps.length === 0}
              className="h-8 px-3 text-xs text-muted hover:text-fg justify-center"
            >
              {clearingAll ? 'Purging...' : 'Purge All App Caches'}
            </Button>
          </div>
        </div>
      </div>

      <div className="flex-1 flex flex-col min-h-0 pt-2">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <Terminal className="w-4 h-4 text-accent" />
            <span className="text-xs font-semibold text-fg">Execution Telemetry Stream</span>
          </div>
          <span className="text-[10px] font-mono text-muted">{logs.length} events logged</span>
        </div>

        <div className="flex-1 min-h-[260px] rounded-xl overflow-hidden border border-border">
          <LogConsole logs={logs} onClear={clearLogs} height="100%" />
        </div>
      </div>
    </div>
  );
}
