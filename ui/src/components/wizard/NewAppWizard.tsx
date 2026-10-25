import { useState } from 'react';
import { Globe, Sparkles } from 'lucide-react';
import { Button, TextInput, Switch, Range, Tabs } from '@aerovex/strata-ui';
import type { InstallRequestDto } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';

export function NewAppWizard() {
  const { installCustom, setActiveTab } = useAppifyStore();

  const [activeTab, setActiveTabStep] = useState<'basics' | 'window' | 'scripts'>('basics');
  const [url, setUrl] = useState('');
  const [name, setName] = useState('');
  const [wmClass, setWmClass] = useState('');
  const [allowedDomains, setAllowedDomains] = useState('');

  const [width, setWidth] = useState(1200);
  const [height, setHeight] = useState(800);
  const [maximize, setMaximize] = useState(false);
  const [zoom, setZoom] = useState(1.0);

  const [singleInstance, setSingleInstance] = useState(true);
  const [hideOnClose, setHideOnClose] = useState(false);
  const [tray, setTray] = useState(false);
  const [startHidden, setStartHidden] = useState(false);
  const [autostart, setAutostart] = useState(false);
  const [autostartHidden, setAutostartHidden] = useState(false);

  const [jsCode, setJsCode] = useState('');
  const [cssCode, setCssCode] = useState('');

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Auto-fill suggested name and wm_class from URL
  const handleUrlChange = (val: string) => {
    setUrl(val);
    setError(null);
    try {
      const parsed = new URL(val.startsWith('http') ? val : `https://${val}`);
      const hostParts = parsed.hostname.split('.');
      const domainName = hostParts.length > 1 ? hostParts[hostParts.length - 2] : hostParts[0];
      if (domainName && !name) {
        const capitalized = domainName.charAt(0).toUpperCase() + domainName.slice(1);
        setName(capitalized);
        setWmClass(`appify-${domainName.toLowerCase()}`);
      }
    } catch {
      // ignore parsing error while typing
    }
  };

  const handleAssemble = async () => {
    if (!url.trim()) {
      setError('Please provide a valid web application URL');
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      let finalUrl = url.trim();
      if (!finalUrl.startsWith('http://') && !finalUrl.startsWith('https://')) {
        finalUrl = `https://${finalUrl}`;
      }

      const domains = allowedDomains
        .split(',')
        .map((d) => d.trim())
        .filter(Boolean);

      const req: InstallRequestDto = {
        url: finalUrl,
        name: name.trim() || undefined,
        wm_class: wmClass.trim() || undefined,
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
        custom_allowed_domains: domains,
        inject_js: jsCode.trim() || undefined,
        inject_css: cssCode.trim() || undefined,
      };

      await installCustom(req);
      setActiveTab('apps');
    } catch (err: any) {
      setError(err?.message || 'Failed to assemble desktop application');
    } finally {
      setSubmitting(false);
    }
  };

  const basicsContent = (
    <div className="space-y-4 pt-1">
      <div className="p-5 sm:p-6 rounded-2xl bg-surface border border-border space-y-4">
        <div>
          <label className="text-xs font-medium text-fg block mb-1.5 flex items-center gap-1.5">
            <Globe className="w-3.5 h-3.5 text-accent" />
            <span>Application URL</span>
          </label>
          <TextInput
            label="Application URL"
            hideLabel
            value={url}
            onChange={handleUrlChange}
            placeholder="https://app.example.com"
            autoFocus
          />
          <span className="text-[11px] text-muted mt-1 block">
            The primary landing address for this isolated desktop application.
          </span>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label className="text-xs font-medium text-fg block mb-1.5">Application Name</label>
            <TextInput
              label="Application Name"
              hideLabel
              value={name}
              onChange={(val: string) => setName(val)}
              placeholder="e.g. Linear, Figma, Discord"
            />
          </div>
          <div>
            <label className="text-xs font-medium text-fg block mb-1.5">Window Class (WM_CLASS)</label>
            <TextInput
              label="Window Class"
              hideLabel
              value={wmClass}
              onChange={(val: string) => setWmClass(val)}
              placeholder="appify-example"
            />
          </div>
        </div>

        <div>
          <label className="text-xs font-medium text-fg block mb-1.5">
            Allowed External Domains (comma-separated)
          </label>
          <TextInput
            label="Allowed External Domains"
            hideLabel
            value={allowedDomains}
            onChange={(val: string) => setAllowedDomains(val)}
            placeholder="e.g. auth.service.com, accounts.google.com"
          />
        </div>
      </div>

      <div className="p-5 sm:p-6 rounded-2xl bg-surface border border-border space-y-4">
        <span className="text-xs font-semibold text-fg block">Process & Behavior Policies</span>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
          <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
            <div>
              <span className="text-xs font-medium text-fg block">Single Instance</span>
              <span className="text-[10px] text-muted">Focus existing window</span>
            </div>
            <Switch checked={singleInstance} onCheckedChange={setSingleInstance} />
          </div>

          <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
            <div>
              <span className="text-xs font-medium text-fg block">Hide on Close</span>
              <span className="text-[10px] text-muted">Minimize to background tray</span>
            </div>
            <Switch checked={hideOnClose} onCheckedChange={setHideOnClose} />
          </div>

          <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
            <div>
              <span className="text-xs font-medium text-fg block">System Tray Icon</span>
              <span className="text-[10px] text-muted">Enable tray menu integration</span>
            </div>
            <Switch checked={tray} onCheckedChange={setTray} />
          </div>

          <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
            <div>
              <span className="text-xs font-medium text-fg block">Start Minimized</span>
              <span className="text-[10px] text-muted">Launch hidden on startup</span>
            </div>
            <Switch checked={startHidden} onCheckedChange={setStartHidden} />
          </div>

          <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
            <div>
              <span className="text-xs font-medium text-fg block">Autostart on Login</span>
              <span className="text-[10px] text-muted">Launch during desktop session</span>
            </div>
            <Switch checked={autostart} onCheckedChange={setAutostart} />
          </div>

          <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
            <div>
              <span className="text-xs font-medium text-fg block">Autostart Hidden</span>
              <span className="text-[10px] text-muted">Autostart silently into tray</span>
            </div>
            <Switch checked={autostartHidden} onCheckedChange={setAutostartHidden} />
          </div>
        </div>
      </div>
    </div>
  );

  const windowContent = (
    <div className="space-y-4 pt-1">
      <div className="p-5 sm:p-6 rounded-2xl bg-surface border border-border space-y-5">
        <span className="text-xs font-semibold text-fg block">Initial Window Dimensions</span>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-xs font-medium text-muted block mb-1.5">Initial Width (px)</label>
            <TextInput
              label="Initial Width"
              hideLabel
              type="number"
              value={width.toString()}
              onChange={(val: string) => setWidth(parseInt(val, 10) || 1200)}
            />
          </div>
          <div>
            <label className="text-xs font-medium text-muted block mb-1.5">Initial Height (px)</label>
            <TextInput
              label="Initial Height"
              hideLabel
              type="number"
              value={height.toString()}
              onChange={(val: string) => setHeight(parseInt(val, 10) || 800)}
            />
          </div>
        </div>

        <div className="flex items-center justify-between p-3 rounded-xl bg-raised border border-border">
          <div>
            <span className="text-xs font-medium text-fg block">Launch Window Maximized</span>
            <span className="text-[10px] text-muted">Default to full display surface on process spawn</span>
          </div>
          <Switch checked={maximize} onCheckedChange={setMaximize} />
        </div>

        <div className="pt-2">
          <Range
            min={0.5}
            max={2.0}
            step={0.05}
            label="Viewport Zoom Factor"
            value={zoom}
            onValueChange={setZoom}
            format={(v) => `${v.toFixed(2)}x`}
          />
        </div>
      </div>
    </div>
  );

  const scriptsContent = (
    <div className="space-y-4 pt-1">
      <div className="p-5 sm:p-6 rounded-2xl bg-surface border border-border space-y-5">
        <div>
          <label className="text-xs font-medium text-fg block mb-1.5">
            Injected UserScript (JavaScript)
          </label>
          <textarea
            value={jsCode}
            onChange={(e) => setJsCode(e.target.value)}
            rows={5}
            placeholder="// Custom JavaScript executed on page load\nconsole.log('Appify injected user script');"
            className="w-full font-mono text-xs p-3.5 rounded-xl bg-raised border border-border text-fg focus:outline-none focus:border-accent resize-none"
          />
        </div>

        <div>
          <label className="text-xs font-medium text-fg block mb-1.5">
            Injected UserStyle (CSS)
          </label>
          <textarea
            value={cssCode}
            onChange={(e) => setCssCode(e.target.value)}
            rows={5}
            placeholder="/* Custom CSS overrides */\nbody { font-feature-settings: 'cv02', 'cv03'; }"
            className="w-full font-mono text-xs p-3.5 rounded-xl bg-raised border border-border text-fg focus:outline-none focus:border-accent resize-none"
          />
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex-1 flex flex-col min-h-0 p-6 sm:p-8 max-w-3xl">
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-fg tracking-tight">Create Desktop Application</h1>
        <p className="text-xs text-muted mt-0.5">
          Transform any public or internal web application into an isolated desktop process with native windowing.
        </p>
      </div>

      <div className="space-y-4 flex-1 overflow-y-auto pr-1">
        <Tabs
          label="New Application Creation Steps"
          value={activeTab}
          onValueChange={(val: string) => setActiveTabStep(val as any)}
          items={[
            { value: 'basics', label: '1. Target & Identity', content: basicsContent },
            { value: 'window', label: '2. Window & Viewport', content: windowContent },
            { value: 'scripts', label: '3. Custom UserScripts', content: scriptsContent },
          ]}
        />

        {error && (
          <div className="p-3.5 rounded-xl bg-red-500/10 border border-red-500/30 text-red-400 text-xs">
            {error}
          </div>
        )}

        <div className="pt-4 border-t border-border flex items-center justify-between">
          <Button variant="secondary" onClick={() => setActiveTab('apps')} className="h-9 px-4 rounded-xl">
            Cancel
          </Button>

          <Button
            variant="primary"
            onClick={handleAssemble}
            disabled={submitting || !url.trim()}
            className="gap-2 font-medium px-4 h-9 rounded-xl shadow-xs"
          >
            <Sparkles className="w-4 h-4" />
            <span>{submitting ? 'Assembling Application...' : 'Assemble Native Application'}</span>
          </Button>
        </div>
      </div>
    </div>
  );
}
