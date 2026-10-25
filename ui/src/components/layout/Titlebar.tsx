import { Settings, HelpCircle, Terminal } from 'lucide-react';
import { AppifyMark, StrataTitlebar, Button, Tooltip } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';

export function Titlebar() {
  const {
    activeTab,
    setActiveTab,
    setSettingsModalOpen,
    setShortcutsModalOpen,
  } = useAppifyStore();

  const brandIcon = (
    <div className="w-6 h-6 rounded-lg bg-raised border border-border flex items-center justify-center shadow-xs">
      <AppifyMark className="w-4 h-4" />
    </div>
  );

  const rightActions = (
    <>
      <Tooltip content="Diagnostics & Event Log">
        <Button
          variant={activeTab === 'diagnostics' ? 'secondary' : 'ghost'}
          onClick={() => setActiveTab('diagnostics')}
          className="h-7 px-2.5 rounded-pill text-xs gap-1.5 text-muted hover:text-fg"
        >
          <Terminal className="w-3.5 h-3.5" />
          <span className="hidden sm:inline font-medium">Diagnostics</span>
        </Button>
      </Tooltip>

      <Tooltip content="Keyboard Shortcuts (?)">
        <Button
          variant="icon"
          onClick={() => setShortcutsModalOpen(true)}
          className="h-7 w-7 text-muted hover:text-fg"
        >
          <HelpCircle className="w-3.5 h-3.5" />
        </Button>
      </Tooltip>

      <Tooltip content="Preferences (Ctrl+,)">
        <Button
          variant="icon"
          onClick={() => setSettingsModalOpen(true)}
          className="h-7 w-7 text-muted hover:text-fg"
        >
          <Settings className="w-3.5 h-3.5" />
        </Button>
      </Tooltip>
    </>
  );

  return (
    <StrataTitlebar
      brandIcon={brandIcon}
      brandName="Appify"
      versionBadge="v0.2.0"
      right={rightActions}
      showWindowControls={true}
    />
  );
}
