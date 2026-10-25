import { useState } from 'react';
import { Search, Plus, RefreshCw } from 'lucide-react';
import { Button, HeroEmptyCard } from '@aerovex/strata-ui';
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
    <div className="flex-1 flex flex-col min-h-0 px-6 py-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-5">
        <div>
          <h1 className="text-lg font-semibold text-fg tracking-tight">Installed Applications</h1>
          <p className="text-xs text-muted">
            Manage, launch, and configure your isolated desktop web applications.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative w-64">
            <Search className="w-3.5 h-3.5 text-muted absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search applications..."
              className="w-full pl-8 pr-3 py-1.5 text-xs rounded-lg bg-surface border border-border text-fg placeholder:text-muted focus:outline-none focus:border-accent transition-colors"
            />
          </div>

          <Button
            variant="ghost"
            onClick={handleRefresh}
            disabled={refreshing}
            className="h-8 w-8 p-0 text-muted hover:text-fg justify-center"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} />
          </Button>

          <Button
            variant="primary"
            onClick={() => setActiveTab('wizard')}
            className="h-8 px-3 text-xs gap-1.5 justify-center"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Create New App</span>
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
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 pb-6">
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
