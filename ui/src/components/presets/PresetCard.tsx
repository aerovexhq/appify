import { useState } from 'react';
import { Download, Play, ExternalLink } from 'lucide-react';
import { Button, StatusBadge } from '@aerovex/strata-ui';
import type { PresetInfoDto } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';

interface PresetCardProps {
  preset: PresetInfoDto;
}

export function PresetCard({ preset }: PresetCardProps) {
  const { installPreset, launchApp } = useAppifyStore();
  const [installing, setInstalling] = useState(false);

  const handleInstall = async () => {
    setInstalling(true);
    try {
      await installPreset(preset.id);
    } finally {
      setInstalling(false);
    }
  };

  const handleOpen = () => {
    if (preset.installed_hash) {
      launchApp(preset.installed_hash);
    }
  };

  return (
    <div className="relative group rounded-2xl bg-surface border border-border hover:border-white/20 p-4.5 transition-all duration-200 flex flex-col justify-between hover:shadow-[0_8px_30px_rgba(0,0,0,0.25)]">
      <div>
        <div className="flex items-start justify-between gap-3 mb-2.5">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-raised border border-border flex items-center justify-center font-bold text-sm text-accent bg-raised shrink-0 shadow-xs">
              {preset.name.charAt(0).toUpperCase()}
            </div>
            <div>
              <h3 className="text-sm font-semibold text-fg group-hover:text-accent transition-colors">
                {preset.name}
              </h3>
              <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-raised text-muted">
                {preset.category}
              </span>
            </div>
          </div>

          {preset.is_installed ? (
            <StatusBadge status="online" label="INSTALLED" />
          ) : (
            <span className="text-[10px] font-mono text-muted px-2 py-0.5 rounded-full bg-raised">
              READY
            </span>
          )}
        </div>

        <p className="text-xs text-muted leading-relaxed line-clamp-2 my-2">
          {preset.description}
        </p>

        <a
          href={preset.url}
          target="_blank"
          rel="noreferrer"
          className="text-[11px] font-mono text-muted/80 hover:text-accent flex items-center gap-1 truncate mt-1"
        >
          <span>{preset.url}</span>
          <ExternalLink className="w-2.5 h-2.5 opacity-50 shrink-0" />
        </a>
      </div>

      <div className="pt-3 border-t border-border mt-3">
        {preset.is_installed ? (
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              onClick={handleOpen}
              className="flex-1 h-8 text-xs gap-1.5 justify-center"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              <span>Open Installed App</span>
            </Button>
          </div>
        ) : (
          <Button
            variant="primary"
            onClick={handleInstall}
            disabled={installing}
            className="w-full h-8 text-xs gap-1.5 font-medium justify-center"
          >
            <Download className="w-3.5 h-3.5" />
            <span>{installing ? 'Installing...' : 'Install Preset'}</span>
          </Button>
        )}
      </div>
    </div>
  );
}
