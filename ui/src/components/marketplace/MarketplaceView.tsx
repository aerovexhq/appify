import { useEffect, useState } from 'react';
import { Download, Check, RefreshCw, Code2, Palette } from 'lucide-react';
import { Button, HeroEmptyCard } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';

export function MarketplaceView() {
  const { catalog, fetchCatalog, installMarketplaceItem, registry } = useAppifyStore();
  const [loading, setLoading] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);

  useEffect(() => {
    if (!catalog) {
      setLoading(true);
      fetchCatalog().finally(() => setLoading(false));
    }
  }, [catalog, fetchCatalog]);

  const handleRefresh = async () => {
    setLoading(true);
    try {
      await fetchCatalog();
    } finally {
      setLoading(false);
    }
  };

  const handleInstall = async (id: string) => {
    setInstallingId(id);
    try {
      await installMarketplaceItem(id);
    } finally {
      setInstallingId(null);
    }
  };

  const installedIds = new Set([
    ...(registry.plugins || []).map((p) => p.id),
    ...(registry.themes || []).map((t) => t.id),
  ]);

  const items = catalog?.items || [];

  return (
    <div className="flex-1 flex flex-col min-h-0 p-6 sm:p-8">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div>
          <div className="flex items-center gap-2.5">
            <h1 className="text-xl font-semibold text-fg tracking-tight">Marketplace Catalog</h1>
            <span className="text-xs font-mono px-2.5 py-0.5 rounded-lg bg-raised border border-accent/40 text-accent font-medium">
              Verified CDN
            </span>
          </div>
          <p className="text-xs text-muted mt-0.5">
            Curated community and official extensions, dark readers, and cosmetic stylesheets.
          </p>
        </div>

        <Button
          variant="ghost"
          onClick={handleRefresh}
          disabled={loading}
          className="h-9 px-4 text-xs gap-2 rounded-xl text-muted hover:text-fg justify-center"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          <span>Refresh Catalog</span>
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto min-h-0 pr-1">
        {items.length === 0 ? (
          <div className="py-12">
            <HeroEmptyCard
              title={loading ? 'Fetching catalog...' : 'Marketplace catalog empty'}
              description={
                loading
                  ? 'Connecting to verified CDN package index...'
                  : 'No public marketplace extensions currently available in this channel.'
              }
              actionLabel="Refresh Catalog"
              onAction={handleRefresh}
              icon="paperplane"
            />
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 pb-8">
            {items.map((item) => {
              const isInstalled = installedIds.has(item.id);
              const isBusy = installingId === item.id;

              return (
                <div
                  key={item.id}
                  className="p-5 sm:p-6 rounded-2xl bg-surface border border-border hover:border-white/20 transition-all flex flex-col justify-between hover:shadow-[0_8px_30px_rgba(0,0,0,0.25)]"
                >
                  <div>
                    <div className="flex items-start justify-between gap-3.5 mb-4">
                      <div className="flex items-center gap-3">
                        <div className="w-11 h-11 rounded-xl bg-raised border border-border flex items-center justify-center shrink-0 shadow-xs">
                          {item.category === 'plugin' ? (
                            <Code2 className="w-4 h-4 text-accent" />
                          ) : (
                            <Palette className="w-4 h-4 text-purple-400" />
                          )}
                        </div>
                        <div>
                          <h3 className="text-base font-semibold text-fg">{item.name}</h3>
                          <div className="flex items-center gap-2 mt-0.5">
                            <span className="text-xs font-mono text-muted">{item.author}</span>
                            <span className="text-xs text-muted">•</span>
                            <span className="text-xs font-mono text-muted">v{item.version}</span>
                          </div>
                        </div>
                      </div>

                      <span className="text-xs font-mono px-2.5 py-0.5 rounded-lg bg-raised border border-border text-muted uppercase">
                        {item.category}
                      </span>
                    </div>

                    <p className="text-xs text-muted leading-relaxed line-clamp-2 my-3">
                      {item.description}
                    </p>
                  </div>

                  <div className="pt-4 border-t border-border mt-4">
                    {isInstalled ? (
                      <div className="flex items-center gap-2 text-xs text-emerald-400 font-medium py-1">
                        <Check className="w-4 h-4" />
                        <span>Installed in Local Registry</span>
                      </div>
                    ) : (
                      <Button
                        variant="primary"
                        onClick={() => handleInstall(item.id)}
                        disabled={isBusy}
                        className="w-full h-9 px-4 text-xs gap-2 font-medium justify-center rounded-xl"
                      >
                        <Download className="w-3.5 h-3.5" />
                        <span>{isBusy ? 'Installing...' : 'Install Extension'}</span>
                      </Button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
