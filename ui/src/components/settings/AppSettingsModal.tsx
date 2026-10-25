import { useState } from 'react';
import { CategorySettingsModal } from '@aerovex/strata-ui';
import { useAppifyStore } from '../../state/useAppifyStore';
import { getAppifySettingsSchema } from '../../config/settingsSchema';

interface AppSettingsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AppSettingsModal({ open, onOpenChange }: AppSettingsModalProps) {
  const { settings, updateSetting, selfInstallBinary } = useAppifyStore();
  const [selfInstalled, setSelfInstalled] = useState(false);

  const handleSelfInstall = async () => {
    try {
      await selfInstallBinary();
      setSelfInstalled(true);
    } catch {
      setSelfInstalled(false);
    }
  };

  const groups = getAppifySettingsSchema(settings, updateSetting, {
    onSelfInstall: handleSelfInstall,
    selfInstalled,
  });

  return (
    <CategorySettingsModal
      open={open}
      onOpenChange={onOpenChange}
      title="Appify Bundler Preferences"
      searchPlaceholder="Search profiles, WebKit engine, proxy, or daemon..."
      groups={groups}
      wide
    />
  );
}
