import { useState } from 'react';
import { HeroEmptyCard, SearchInput, Button } from '@aerovex/strata-ui';
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

        <div className="w-72">
          <SearchInput
            value={search}
            onChange={setSearch}
            placeholder="Search preset catalog..."
          />
        </div>
      </div>

      <div className="flex items-center gap-2 overflow-x-auto pb-4 mb-4 shrink-0">
        {CATEGORIES.map((cat) => {
          const isSelected = selectedCategory === cat;
          return (
            <Button
              key={cat}
              variant={isSelected ? "secondary" : "ghost"}
              size="sm"
              onClick={() => setSelectedCategory(cat)}
              className={isSelected ? "font-semibold border-white/20" : ""}
            >
              {cat}
            </Button>
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
