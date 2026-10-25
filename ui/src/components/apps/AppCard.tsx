import { useState } from 'react';
import { Play, Settings, FolderOpen, Trash2, RotateCcw, ExternalLink } from 'lucide-react';
import { Button, StatusBadge, Tooltip } from '@aerovex/strata-ui';
import type { AppInfoDto } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';

interface AppCardProps {
  app: AppInfoDto;
  onEdit: (app: AppInfoDto) => void;
}

export function AppCard({ app, onEdit }: AppCardProps) {
  const { launchApp, uninstallApp, clearCache, openFolder } = useAppifyStore();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [busy, setBusy] = useState(false);

  const handleLaunch = async () => {
    setBusy(true);
    try {
      await launchApp(app.hash);
    } finally {
      setBusy(false);
    }
  };

  const handleUninstall = async () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      setTimeout(() => setConfirmDelete(false), 4000);
      return;
    }
    setBusy(true);
    try {
      await uninstallApp(app.hash);
    } finally {
      setBusy(false);
      setConfirmDelete(false);
    }
  };

  const handleClearCache = async () => {
    setBusy(true);
    try {
      await clearCache(app.hash);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="relative group rounded-xl bg-surface border border-border hover:border-white/15 p-4 transition-all duration-200 flex flex-col justify-between hover:shadow-[0_4px_24px_rgba(0,0,0,0.3)]">
      <div>
        <div className="flex items-start justify-between gap-3 mb-3">
          <div className="flex items-center gap-3">
            <div className="w-11 h-11 rounded-lg bg-raised border border-border flex items-center justify-center overflow-hidden shrink-0 group-hover:border-white/20 transition-colors">
              {app.icon_base64 ? (
                <img
                  src={`data:image/png;base64,${app.icon_base64}`}
                  alt={app.name}
                  className="w-full h-full object-cover"
                />
              ) : app.icon_path ? (
                <img
                  src={app.icon_path}
                  alt={app.name}
                  className="w-full h-full object-cover"
                  onError={(e) => {
                    (e.target as HTMLElement).style.display = 'none';
                  }}
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center font-bold text-sm text-accent bg-accent/10">
                  {app.name.charAt(0).toUpperCase()}
                </div>
              )}
            </div>

            <div className="min-w-0">
              <h3 className="text-sm font-semibold text-fg group-hover:text-accent transition-colors truncate">
                {app.name}
              </h3>
              <a
                href={app.url}
                target="_blank"
                rel="noreferrer"
                className="text-xs font-mono text-muted hover:text-accent flex items-center gap-1 truncate max-w-[200px]"
              >
                <span>{app.url}</span>
                <ExternalLink className="w-2.5 h-2.5 opacity-50 shrink-0" />
              </a>
            </div>
          </div>

          <StatusBadge
            status={app.is_running ? 'online' : 'standby'}
            label={app.is_running ? 'RUNNING' : 'SEALED'}
          />
        </div>

        <div className="flex flex-wrap gap-1.5 my-3">
          <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-raised text-muted">
            {app.wm_class}
          </span>
          {app.autostart && (
            <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-accent/15 text-accent font-medium">
              Autostart
            </span>
          )}
          {app.tray && (
            <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-raised text-muted">
              Tray
            </span>
          )}
          {app.has_custom_scripts && (
            <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 font-medium">
              UserScripts
            </span>
          )}
        </div>
      </div>

      <div className="pt-3 border-t border-border flex items-center justify-between gap-2 mt-2">
        <Button
          variant="secondary"
          onClick={handleLaunch}
          disabled={busy}
          className="flex-1 h-8 text-xs gap-1.5 font-medium justify-center hover:text-accent hover:border-accent"
        >
          <Play className="w-3.5 h-3.5 fill-current text-accent" />
          <span>Launch</span>
        </Button>

        <div className="flex items-center gap-1">
          <Tooltip content="Edit Application Settings">
            <Button
              variant="ghost"
              onClick={() => onEdit(app)}
              className="h-8 w-8 p-0 text-muted hover:text-fg justify-center"
            >
              <Settings className="w-3.5 h-3.5" />
            </Button>
          </Tooltip>

          <Tooltip content="Open Profile Directory">
            <Button
              variant="ghost"
              onClick={() => openFolder(app.hash)}
              className="h-8 w-8 p-0 text-muted hover:text-fg justify-center"
            >
              <FolderOpen className="w-3.5 h-3.5" />
            </Button>
          </Tooltip>

          <Tooltip content="Clear Profile Disk Cache">
            <Button
              variant="ghost"
              onClick={handleClearCache}
              className="h-8 w-8 p-0 text-muted hover:text-fg justify-center"
            >
              <RotateCcw className="w-3.5 h-3.5" />
            </Button>
          </Tooltip>

          <Tooltip content={confirmDelete ? 'Click again to confirm purge' : 'Uninstall Application'}>
            <Button
              variant={confirmDelete ? 'secondary' : 'ghost'}
              onClick={handleUninstall}
              className={`h-8 px-2 text-xs transition-colors justify-center ${
                confirmDelete ? 'bg-red-500/20 text-red-400 border border-red-500/40' : 'text-muted hover:text-red-400'
              }`}
            >
              <Trash2 className="w-3.5 h-3.5" />
              {confirmDelete && <span className="ml-1 text-[10px]">Confirm?</span>}
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>
  );
}
