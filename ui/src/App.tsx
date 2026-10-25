import { useEffect, useMemo } from 'react';
import { Titlebar } from './components/layout/Titlebar';
import { Sidebar } from './components/layout/Sidebar';
import { MetricsHeader } from './components/layout/MetricsHeader';
import { AppGrid } from './components/apps/AppGrid';
import { PresetStore } from './components/presets/PresetStore';
import { NewAppWizard } from './components/wizard/NewAppWizard';
import { ExtensionsView } from './components/extensions/ExtensionsView';
import { MarketplaceView } from './components/marketplace/MarketplaceView';
import { DiagnosticsView } from './components/diagnostics/DiagnosticsView';
import { AppSettingsModal } from './components/settings/AppSettingsModal';
import {
  ShortcutLegendModal,
  useKeyboardShortcuts,
  type ShortcutDefinition,
} from '@aerovex/strata-ui';
import { useAppifyStore } from './state/useAppifyStore';

export function App() {
  const {
    activeTab,
    setActiveTab,
    settingsModalOpen,
    setSettingsModalOpen,
    shortcutsModalOpen,
    setShortcutsModalOpen,
    loadAll,
  } = useAppifyStore();

  useEffect(() => {
    loadAll();
  }, [loadAll]);

  const shortcutDefs = useMemo<ShortcutDefinition[]>(
    () => [
      {
        id: 'nav.apps',
        category: 'Navigation',
        description: 'View Installed Applications',
        defaultKey: '1',
        action: () => setActiveTab('apps'),
      },
      {
        id: 'nav.presets',
        category: 'Navigation',
        description: 'View Curated Presets',
        defaultKey: '2',
        action: () => setActiveTab('presets'),
      },
      {
        id: 'nav.wizard',
        category: 'Navigation',
        description: 'Create New Application Wizard',
        defaultKey: 'N',
        action: () => setActiveTab('wizard'),
      },
      {
        id: 'nav.diagnostics',
        category: 'Navigation',
        description: 'Open Diagnostics & Event Stream',
        defaultKey: 'D',
        action: () => setActiveTab('diagnostics'),
      },
      {
        id: 'app.settings',
        category: 'General',
        description: 'Open Bundler Settings',
        defaultKey: ',',
        action: () => setSettingsModalOpen(true),
      },
      {
        id: 'app.shortcuts',
        category: 'General',
        description: 'Toggle Keyboard Shortcuts Legend',
        defaultKey: '?',
        action: () => setShortcutsModalOpen(true),
      },
    ],
    [setActiveTab, setSettingsModalOpen, setShortcutsModalOpen]
  );

  useKeyboardShortcuts(shortcutDefs);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-bg text-fg select-none font-sans">
      <Titlebar />

      <div className="flex flex-1 min-h-0 overflow-hidden">
        <Sidebar />

        <main className="flex-1 flex flex-col min-h-0 overflow-hidden bg-canvas">
          <MetricsHeader />

          <div className="flex-1 flex min-h-0 overflow-hidden">
            {activeTab === 'apps' && <AppGrid />}
            {activeTab === 'presets' && <PresetStore />}
            {activeTab === 'wizard' && <NewAppWizard />}
            {activeTab === 'extensions' && <ExtensionsView />}
            {activeTab === 'marketplace' && <MarketplaceView />}
            {activeTab === 'diagnostics' && <DiagnosticsView />}
          </div>
        </main>
      </div>

      <AppSettingsModal
        open={settingsModalOpen}
        onOpenChange={setSettingsModalOpen}
      />

      <ShortcutLegendModal
        open={shortcutsModalOpen}
        onClose={() => setShortcutsModalOpen(false)}
        definitions={shortcutDefs}
        title="Appify Keyboard Shortcuts"
      />
    </div>
  );
}
