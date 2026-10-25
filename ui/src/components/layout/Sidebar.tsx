import {
  LayoutGrid,
  Sparkles,
  PlusCircle,
  Puzzle,
  ShoppingBag,
  Activity,
  Settings,
} from 'lucide-react';
import type { ActiveTab } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';

interface NavItem {
  id: ActiveTab;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  countKey?: 'apps' | 'presets';
}

const NAV_ITEMS: NavItem[] = [
  { id: 'apps', label: 'My Applications', icon: LayoutGrid, countKey: 'apps' },
  { id: 'presets', label: 'Curated Presets', icon: Sparkles, countKey: 'presets' },
  { id: 'wizard', label: 'Create New App', icon: PlusCircle },
  { id: 'extensions', label: 'UserScripts & Themes', icon: Puzzle },
  { id: 'marketplace', label: 'Marketplace', icon: ShoppingBag },
  { id: 'diagnostics', label: 'Diagnostics & Logs', icon: Activity },
];

export function Sidebar() {
  const { activeTab, setActiveTab, apps, presets, setSettingsModalOpen } = useAppifyStore();

  return (
    <aside className="w-56 shrink-0 bg-surface/50 border-r border-border flex flex-col justify-between p-3 select-none">
      <div className="space-y-1">
        <div className="px-2 py-1.5 text-[10px] font-mono tracking-wider text-muted uppercase">
          Navigation
        </div>
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;
          const count = item.countKey === 'apps' ? apps.length : item.countKey === 'presets' ? presets.length : null;

          return (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center justify-between px-2.5 py-2 rounded-lg text-xs font-medium transition-all ${
                isActive
                  ? 'bg-accent/15 text-accent border border-accent/40 shadow-sm'
                  : 'text-muted hover:text-fg hover:bg-raised/70 border border-transparent'
              }`}
            >
              <div className="flex items-center gap-2.5">
                <Icon className={`w-4 h-4 ${isActive ? 'text-accent' : 'text-muted'}`} />
                <span>{item.label}</span>
              </div>
              {count !== null && (
                <span
                  className={`text-[10px] font-mono px-1.5 py-0.2 rounded-full ${
                    isActive ? 'bg-accent/25 text-accent' : 'bg-raised text-muted'
                  }`}
                >
                  {count}
                </span>
              )}
            </button>
          );
        })}
      </div>

      <div className="pt-3 border-t border-border space-y-1">
        <button
          onClick={() => setSettingsModalOpen(true)}
          className="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-xs text-muted hover:text-fg hover:bg-raised/70 transition-colors"
        >
          <Settings className="w-4 h-4 text-muted" />
          <span>Bundler Settings</span>
        </button>
        <div className="px-2.5 pt-2 flex items-center justify-between text-[10px] text-muted font-mono">
          <span>Profile Partitioning</span>
          <span className="text-accent font-semibold">Active</span>
        </div>
      </div>
    </aside>
  );
}
