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
    <div className="flex-1 flex flex-col min-h-0 p-6 sm:p-8 space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-fg tracking-tight">Diagnostics & Runtime Environment</h1>
        <p className="text-xs text-muted mt-0.5">
          System paths, WebKit profile data directories, execution event telemetry, and cache maintenance.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="p-5 sm:p-6 rounded-2xl bg-surface border border-border space-y-4">
          <div className="flex items-center gap-2.5">
            <HardDrive className="w-4 h-4 text-accent" />
            <span className="text-sm font-semibold text-fg">Host File System Topology</span>
          </div>

          <div className="space-y-3 text-xs font-mono">
            <div>
              <span className="text-muted block text-xs mb-1">Profiles Root</span>
              <span className="text-fg truncate block bg-raised px-3 py-1.5 rounded-lg border border-border">
                {systemInfo?.profiles_dir || '~/.local/share/appify/profiles'}
              </span>
            </div>
            <div>
              <span className="text-muted block text-xs mb-1">Desktop Applications (.desktop)</span>
              <span className="text-fg truncate block bg-raised px-3 py-1.5 rounded-lg border border-border">
                {systemInfo?.applications_dir || '~/.local/share/applications'}
              </span>
            </div>
            <div>
              <span className="text-muted block text-xs mb-1">Session Autostart Entries</span>
              <span className="text-fg truncate block bg-raised px-3 py-1.5 rounded-lg border border-border">
                {systemInfo?.autostart_dir || '~/.config/autostart'}
              </span>
            </div>
          </div>
        </div>

        <div className="p-5 sm:p-6 rounded-2xl bg-surface border border-border flex flex-col justify-between">
          <div className="space-y-2.5">
            <div className="flex items-center gap-2.5">
              <Shield className="w-4 h-4 text-accent" />
              <span className="text-sm font-semibold text-fg">System Binary & Maintenance</span>
            </div>
            <p className="text-xs text-muted leading-relaxed">
              Install the Appify CLI binary to <code className="text-fg font-mono">/usr/local/bin/appify</code> to invoke headless application packaging and WebKit profile management from your terminal.
            </p>
          </div>

          <div className="flex flex-wrap gap-3 pt-4 border-t border-border mt-4">
            <Button
              variant={installedPath ? 'secondary' : 'primary'}
              onClick={handleSelfInstall}
              disabled={installing}
              className="h-9 px-4 text-xs gap-2 font-medium justify-center rounded-xl"
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
              className="h-9 px-4 text-xs rounded-xl text-muted hover:text-fg justify-center"
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
