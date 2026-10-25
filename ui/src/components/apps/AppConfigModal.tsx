import { useState, useEffect } from 'react';
import {
  Modal,
  Button,
  TextInput,
  Switch,
  Range,
  Tabs,
} from '@aerovex/strata-ui';
import type { AppInfoDto, UpdateConfigDto, ScriptsDto } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';
import { bridge } from '../../transport/bridge';

interface AppConfigModalProps {
  app: AppInfoDto | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AppConfigModal({ app, open, onOpenChange }: AppConfigModalProps) {
  const { updateAppConfig } = useAppifyStore();

  const [activeTab, setActiveTab] = useState<'general' | 'window' | 'scripts'>('general');
  const [name, setName] = useState('');
  const [wmClass, setWmClass] = useState('');
  const [width, setWidth] = useState<number>(1200);
  const [height, setHeight] = useState<number>(800);
  const [maximize, setMaximize] = useState(false);
  const [zoom, setZoom] = useState<number>(1.0);
  const [singleInstance, setSingleInstance] = useState(true);
  const [hideOnClose, setHideOnClose] = useState(false);
  const [tray, setTray] = useState(false);
  const [startHidden, setStartHidden] = useState(false);
  const [autostart, setAutostart] = useState(false);
  const [autostartHidden, setAutostartHidden] = useState(false);
  const [customAllowedDomains, setCustomAllowedDomains] = useState<string>('');

  const [jsCode, setJsCode] = useState('');
  const [cssCode, setCssCode] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (app) {
      setName(app.name || '');
      setWmClass(app.wm_class || '');
      setWidth(app.width || 1200);
      setHeight(app.height || 800);
      setMaximize(!!app.maximize);
      setZoom(app.zoom || 1.0);
      setSingleInstance(app.single_instance ?? true);
      setHideOnClose(app.hide_on_close ?? false);
      setTray(app.tray ?? false);
      setStartHidden(app.start_hidden ?? false);
      setAutostart(app.autostart ?? false);
      setAutostartHidden(app.autostart_hidden ?? false);
      setCustomAllowedDomains((app.custom_allowed_domains || []).join(', '));

      // Fetch scripts
      bridge
        .getAppScripts(app.hash)
        .then((scripts: ScriptsDto) => {
          setJsCode(scripts.js || '');
          setCssCode(scripts.css || '');
        })
        .catch(() => {
          setJsCode('');
          setCssCode('');
        });
    }
  }, [app]);

  if (!app) return null;

  const handleSave = async () => {
    setSaving(true);
    try {
      const allowedDomains = customAllowedDomains
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean);

      const updateReq: UpdateConfigDto = {
        name,
        wm_class: wmClass,
        width,
        height,
        maximize,
        zoom,
        single_instance: singleInstance,
        hide_on_close: hideOnClose,
        tray,
        start_hidden: startHidden,
        autostart,
        autostart_hidden: autostartHidden,
        custom_allowed_domains: allowedDomains,
        inject_js: jsCode || null,
        inject_css: cssCode || null,
      };

      await updateAppConfig(app.hash, updateReq);
      await bridge.saveAppScripts(app.hash, jsCode, cssCode, [], []);
      onOpenChange(false);
    } finally {
      setSaving(false);
    }
  };

  const generalContent = (
    <div className="space-y-3.5 pt-1">
      <div>
        <label className="text-xs font-medium text-muted block mb-1">Application Title</label>
        <TextInput
          label="Application Title"
          hideLabel
          value={name}
          onChange={(val: string) => setName(val)}
        />
      </div>

      <div>
        <label className="text-xs font-medium text-muted block mb-1">Window Class (WM_CLASS)</label>
        <TextInput
          label="Window Class"
          hideLabel
          value={wmClass}
          onChange={(val: string) => setWmClass(val)}
        />
      </div>

      <div>
        <label className="text-xs font-medium text-muted block mb-1">Allowed External Domains (comma-separated)</label>
        <TextInput
          label="Allowed External Domains"
          hideLabel
          value={customAllowedDomains}
          onChange={(val: string) => setCustomAllowedDomains(val)}
          placeholder="e.g. auth.service.com, cdn.service.com"
        />
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
        <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
          <span className="text-xs text-fg">Single Instance</span>
          <Switch checked={singleInstance} onCheckedChange={setSingleInstance} />
        </div>
        <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
          <span className="text-xs text-fg">Hide Window on Close</span>
          <Switch checked={hideOnClose} onCheckedChange={setHideOnClose} />
        </div>
        <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
          <span className="text-xs text-fg">System Tray Icon</span>
          <Switch checked={tray} onCheckedChange={setTray} />
        </div>
        <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
          <span className="text-xs text-fg">Start Minimized / Hidden</span>
          <Switch checked={startHidden} onCheckedChange={setStartHidden} />
        </div>
        <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
          <span className="text-xs text-fg">Autostart on Login</span>
          <Switch checked={autostart} onCheckedChange={setAutostart} />
        </div>
        <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
          <span className="text-xs text-fg">Autostart Hidden in Tray</span>
          <Switch checked={autostartHidden} onCheckedChange={setAutostartHidden} />
        </div>
      </div>
    </div>
  );

  const windowContent = (
    <div className="space-y-4 pt-1">
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="text-xs font-medium text-muted block mb-1">Initial Width (px)</label>
          <TextInput
            label="Initial Width"
            hideLabel
            type="number"
            value={width.toString()}
            onChange={(val: string) => setWidth(parseInt(val, 10) || 1200)}
          />
        </div>
        <div>
          <label className="text-xs font-medium text-muted block mb-1">Initial Height (px)</label>
          <TextInput
            label="Initial Height"
            hideLabel
            type="number"
            value={height.toString()}
            onChange={(val: string) => setHeight(parseInt(val, 10) || 800)}
          />
        </div>
      </div>

      <div className="flex items-center justify-between p-2.5 rounded-lg bg-raised/50 border border-border">
        <div>
          <span className="text-xs text-fg block font-medium">Launch Maximized</span>
          <span className="text-[11px] text-muted">Span window across entire display surface</span>
        </div>
        <Switch checked={maximize} onCheckedChange={setMaximize} />
      </div>

      <div>
        <Range
          min={0.5}
          max={2.0}
          step={0.05}
          label="Viewport Zoom Factor"
          value={zoom}
          onValueChange={(val: number) => setZoom(val)}
          format={(v) => `${v.toFixed(2)}x`}
        />
      </div>
    </div>
  );

  const scriptsContent = (
    <div className="space-y-3 pt-1">
      <div>
        <label className="text-xs font-medium text-muted block mb-1.5">
          Custom UserScript (JavaScript injected on DOM load)
        </label>
        <textarea
          value={jsCode}
          onChange={(e) => setJsCode(e.target.value)}
          rows={6}
          placeholder="// Custom DOM manipulation script\nconsole.log('Appify injected script');"
          className="w-full font-mono text-xs p-3 rounded-lg bg-raised border border-border text-fg focus:outline-none focus:border-accent resize-none"
        />
      </div>

      <div>
        <label className="text-xs font-medium text-muted block mb-1.5">
          Custom UserStyle (CSS stylesheets injected into WebKit viewport)
        </label>
        <textarea
          value={cssCode}
          onChange={(e) => setCssCode(e.target.value)}
          rows={6}
          placeholder="/* Custom CSS overrides */\nbody { font-family: Inter, sans-serif; }"
          className="w-full font-mono text-xs p-3 rounded-lg bg-raised border border-border text-fg focus:outline-none focus:border-accent resize-none"
        />
      </div>
    </div>
  );

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title={`Configure "${app.name}"`}
      className="max-w-2xl"
    >
      <div className="space-y-4 pt-1">
        <Tabs
          label="Configuration Categories"
          value={activeTab}
          onValueChange={(val: string) => setActiveTab(val as any)}
          items={[
            { value: 'general', label: 'General & Behavior', content: generalContent },
            { value: 'window', label: 'Window & Display', content: windowContent },
            { value: 'scripts', label: 'Custom Scripts & CSS', content: scriptsContent },
          ]}
        />

        <div className="pt-4 border-t border-border flex justify-end gap-2">
          <Button variant="secondary" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSave} disabled={saving}>
            {saving ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
