import { useState } from 'react';
import { Search } from 'lucide-react';
import { HeroEmptyCard } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';
import { PresetCard } from './PresetCard';

const CATEGORIES = ['All', 'Productivity', 'Developer', 'Communication', 'Media', 'Social'];

export function PresetStore() {
  const { presets } = useAppifyStore();
  const [search, setSearch] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');

  const filteredPresets = presets.filter((p) => {
    const matchesSearch =
      !search ||
      p.name.toLowerCase().includes(search.toLowerCase()) ||
      p.description.toLowerCase().includes(search.toLowerCase()) ||
      p.url.toLowerCase().includes(search.toLowerCase());

    const matchesCategory =
      selectedCategory === 'All' ||
      p.category.toLowerCase() === selectedCategory.toLowerCase();

    return matchesSearch && matchesCategory;
  });

  return (
    <div className="flex-1 flex flex-col min-h-0 p-6 sm:p-8">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div>
          <div className="flex items-center gap-2.5">
            <h1 className="text-xl font-semibold text-fg tracking-tight">Curated Presets</h1>
            <span className="text-xs font-mono px-2.5 py-0.5 rounded-lg bg-raised border border-accent/40 text-accent font-medium">
              Verified Profiles
            </span>
          </div>
          <p className="text-xs text-muted mt-0.5">
            Pre-configured templates with high-fidelity icons, WM classes, and optimal WebKit partition settings.
          </p>
        </div>

        <div className="relative w-72">
          <Search className="w-3.5 h-3.5 text-muted absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search preset catalog..."
            className="w-full h-9 pl-9 pr-4 text-xs rounded-xl bg-raised border border-border text-fg placeholder:text-muted focus:outline-none focus:border-white/30 transition-colors"
          />
        </div>
      </div>

      <div className="flex items-center gap-2 overflow-x-auto pb-4 mb-4 shrink-0">
        {CATEGORIES.map((cat) => {
          const isSelected = selectedCategory === cat;
          return (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={`px-3.5 py-1.5 rounded-xl text-xs font-medium cursor-pointer transition-colors ${
                isSelected
                  ? 'bg-raised text-fg shadow-xs font-semibold border border-white/20'
                  : 'text-muted hover:text-fg hover:bg-raised border border-transparent'
              }`}
            >
              {cat}
            </button>
          );
        })}
      </div>

      <div className="flex-1 overflow-y-auto min-h-0 pr-1">
        {filteredPresets.length === 0 ? (
          <div className="py-12">
            <HeroEmptyCard
              title="No presets found"
              description={`No curated preset matched "${search || selectedCategory}". Try a different search term or category.`}
              actionLabel="Reset Category Filter"
              onAction={() => {
                setSearch('');
                setSelectedCategory('All');
              }}
              icon="puzzle"
            />
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 pb-8">
            {filteredPresets.map((preset) => (
              <PresetCard key={preset.id} preset={preset} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
