import { MetricCard } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';

export function MetricsHeader() {
  const { apps, presets, registry } = useAppifyStore();

  const totalScripts = (registry.plugins?.length || 0) + (registry.themes?.length || 0);

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 px-6 pt-5 pb-2">
      <MetricCard
        title="INSTALLED APPLICATIONS"
        value={apps.length}
        unit="apps"
        history={[Math.max(0, apps.length - 2), Math.max(0, apps.length - 1), apps.length]}
        status="nominal"
      />
      <MetricCard
        title="CURATED PRESETS"
        value={presets.length}
        unit="ready"
        history={[presets.length, presets.length, presets.length]}
        status="nominal"
      />
      <MetricCard
        title="SCRIPTS & THEMES"
        value={totalScripts}
        unit="active"
        history={[0, 1, totalScripts]}
        status="nominal"
      />
      <MetricCard
        title="STORAGE PARTITIONING"
        value="SHA-256"
        unit="isolated"
        status="nominal"
      />
    </div>
  );
}
