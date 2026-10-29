import { useState } from 'react';
import { Plus, RefreshCw } from 'lucide-react';
import { Button, HeroEmptyCard, SearchInput } from '@aerovex/strata-ui';
import type { AppInfoDto } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';
import { AppCard } from './AppCard';
import { AppConfigModal } from './AppConfigModal';

export function AppGrid() {
  const { apps, searchQuery, setSearchQuery, setActiveTab, fetchApps } = useAppifyStore();
  const [editingApp, setEditingApp] = useState<AppInfoDto | null>(null);
  const [modalOpen, setModalOpen] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  const filteredApps = apps.filter((app) => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    return (
      app.name.toLowerCase().includes(q) ||
      app.url.toLowerCase().includes(q) ||
      app.wm_class.toLowerCase().includes(q)
    );
  });

  const handleEdit = (app: AppInfoDto) => {
    setEditingApp(app);
    setModalOpen(true);
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await fetchApps();
    } finally {
      setRefreshing(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col min-h-0 p-6 sm:p-8">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div>
          <h1 className="text-xl font-semibold text-fg tracking-tight">Installed Applications</h1>
          <p className="text-xs text-muted mt-0.5">
            Manage, launch, and configure your isolated desktop web applications.
          </p>
        </div>

        <div className="flex items-center gap-2.5">
          <div className="w-72">
            <SearchInput
              value={searchQuery}
              onChange={setSearchQuery}
              placeholder="Search applications..."
            />
          </div>

          <Button
            variant="ghost"
            onClick={handleRefresh}
            disabled={refreshing}
            className="h-9 w-9 p-0 rounded-xl text-muted hover:text-fg justify-center"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} />
          </Button>

          <Button
            variant="primary"
            onClick={() => setActiveTab('wizard')}
            className="h-9 px-4 text-xs rounded-xl font-medium gap-2 justify-center shadow-xs"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>New App</span>
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto min-h-0 pr-1">
        {filteredApps.length === 0 ? (
          <div className="py-12">
            <HeroEmptyCard
              title={searchQuery ? 'No matching applications' : 'No applications installed yet'}
              description={
                searchQuery
                  ? `No applications matched "${searchQuery}". Clear your search query or create a new desktop wrapper.`
                  : 'Transform any web application into a dedicated, isolated desktop application with WebKit profile partitioning.'
              }
              actionLabel={searchQuery ? 'Clear Search Filter' : 'Browse Curated Presets'}
              onAction={() => {
                if (searchQuery) {
                  setSearchQuery('');
                } else {
                  setActiveTab('presets');
                }
              }}
              icon="puzzle"
            />
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 pb-8">
            {filteredApps.map((app) => (
              <AppCard key={app.hash} app={app} onEdit={handleEdit} />
            ))}
          </div>
        )}
      </div>

      <AppConfigModal
        app={editingApp}
        open={modalOpen}
        onOpenChange={setModalOpen}
      />
    </div>
  );
}
