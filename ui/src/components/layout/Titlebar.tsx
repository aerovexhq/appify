import { Layers, Settings, HelpCircle, Terminal } from 'lucide-react';
import { Button, Tooltip } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';

export function Titlebar() {
  const {
    activeTab,
    setActiveTab,
    setSettingsModalOpen,
    setShortcutsModalOpen,
  } = useAppifyStore();

  return (
    <header
      data-tauri-drag-region
      className="h-11 shrink-0 bg-surface/80 backdrop-blur-md border-b border-border flex items-center justify-between px-3 select-none z-30"
    >
      <div className="flex items-center gap-2.5" data-tauri-drag-region>
        <div className="w-5 h-5 rounded-md bg-accent/20 border border-accent/40 flex items-center justify-center">
          <Layers className="w-3 h-3 text-accent" />
        </div>
        <div className="flex items-baseline gap-2">
          <span className="font-semibold text-xs tracking-wider text-fg uppercase">
            Appify
          </span>
          <span className="text-[10px] text-muted font-mono px-1.5 py-0.5 rounded bg-raised border border-border">
            v0.2.0
          </span>
        </div>
      </div>

      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        <Tooltip content="Diagnostics & Execution Log">
          <Button
            variant={activeTab === 'diagnostics' ? 'secondary' : 'ghost'}
            onClick={() => setActiveTab('diagnostics')}
            className="h-7 px-2 text-xs gap-1.5 text-muted hover:text-fg"
          >
            <Terminal className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Diagnostics</span>
          </Button>
        </Tooltip>

        <Tooltip content="Keyboard Shortcuts (?)">
          <Button
            variant="ghost"
            onClick={() => setShortcutsModalOpen(true)}
            className="h-7 w-7 p-0 text-muted hover:text-fg"
          >
            <HelpCircle className="w-3.5 h-3.5" />
          </Button>
        </Tooltip>

        <Tooltip content="Preferences (Ctrl+,)">
          <Button
            variant="ghost"
            onClick={() => setSettingsModalOpen(true)}
            className="h-7 w-7 p-0 text-muted hover:text-fg"
          >
            <Settings className="w-3.5 h-3.5" />
          </Button>
        </Tooltip>
      </div>
    </header>
  );
}
