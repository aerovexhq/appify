import { useState } from 'react';
import { Plus, Trash2, Code2, Palette, ShieldCheck } from 'lucide-react';
import { Button, Modal, TextInput, Tabs, HeroEmptyCard } from '@aerovex/strata-ui';
import type { ExtensionItem } from '../../types';
import { useAppifyStore } from '../../state/useAppifyStore';
import { bridge } from '../../transport/bridge';

export function ExtensionsView() {
  const { registry, fetchRegistry, addLog } = useAppifyStore();
  const [activeTab, setActiveTab] = useState<'plugins' | 'themes'>('plugins');
  const [addModalOpen, setAddModalOpen] = useState(false);

  // New extension form
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [author, setAuthor] = useState('');
  const [version, setVersion] = useState('1.0.0');
  const [content, setContent] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const items = activeTab === 'plugins' ? registry.plugins : registry.themes;

  const handleDelete = async (id: string) => {
    try {
      await bridge.deleteRegistryItem(id);
      addLog('info', `Deleted extension "${id}".`);
      await fetchRegistry();
    } catch (err: any) {
      addLog('error', `Failed to delete extension: ${err?.message}`);
    }
  };

  const handleCreate = async () => {
    if (!name.trim() || !content.trim()) return;

    setSubmitting(true);
    try {
      const id = name.toLowerCase().replace(/[^a-z0-9]+/g, '-');
      const item: ExtensionItem = {
        id,
        name: name.trim(),
        description: description.trim() || 'Custom user extension',
        category: activeTab === 'plugins' ? 'plugin' : 'theme',
        content: content.trim(),
        author: author.trim() || 'Local Operator',
        version: version.trim() || '1.0.0',
        is_builtin: false,
      };

      await bridge.saveRegistryItem(item);
      addLog('info', `Registered new ${item.category} "${item.name}".`);
      await fetchRegistry();
      setAddModalOpen(false);
      setName('');
      setDescription('');
      setContent('');
    } catch (err: any) {
      addLog('error', `Failed to register extension: ${err?.message}`);
    } finally {
      setSubmitting(false);
    }
  };

  const renderContent = (
    <div className="flex-1 overflow-y-auto min-h-0 pr-1">
      {(!items || items.length === 0) ? (
        <div className="py-12">
          <HeroEmptyCard
            title={`No ${activeTab === 'plugins' ? 'plugins' : 'themes'} installed`}
            description={`Create your first custom ${activeTab === 'plugins' ? 'JavaScript extension' : 'CSS theme'} or install one from the marketplace.`}
            actionLabel={`Create ${activeTab === 'plugins' ? 'Plugin' : 'Theme'}`}
            onAction={() => setAddModalOpen(true)}
            icon="puzzle"
          />
        </div>
      ) : (
        <div className="space-y-3 pb-6">
          {items.map((item) => (
            <div
              key={item.id}
              className="p-4 rounded-xl bg-surface border border-border flex flex-col justify-between gap-3"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div className="w-9 h-9 rounded-lg bg-raised border border-border flex items-center justify-center shrink-0">
                    {item.category === 'plugin' ? (
                      <Code2 className="w-4 h-4 text-accent" />
                    ) : (
                      <Palette className="w-4 h-4 text-purple-400" />
                    )}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="text-sm font-semibold text-fg">{item.name}</h3>
                      <span className="text-[10px] font-mono text-muted px-2 py-0.5 rounded-full bg-raised">
                        v{item.version}
                      </span>
                      {item.is_builtin && (
                        <span className="text-[10px] font-mono text-accent px-2 py-0.5 rounded-full bg-raised flex items-center gap-1 font-medium">
                          <ShieldCheck className="w-2.5 h-2.5" />
                          <span>Built-in</span>
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-muted mt-0.5">{item.description}</p>
                  </div>
                </div>

                {!item.is_builtin && (
                  <Button
                    variant="ghost"
                    onClick={() => handleDelete(item.id)}
                    className="h-8 w-8 p-0 text-muted hover:text-red-400 justify-center"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                )}
              </div>

              <div className="rounded-lg bg-raised border border-border p-3 max-h-36 overflow-y-auto font-mono text-[11px] text-muted leading-relaxed">
                <pre className="whitespace-pre-wrap">{item.content}</pre>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );

  return (
    <div className="flex-1 flex flex-col min-h-0 px-6 py-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
        <div>
          <h1 className="text-lg font-semibold text-fg tracking-tight">Extensions & UserScripts</h1>
          <p className="text-xs text-muted">
            Reusable JavaScript extensions and CSS themes injected into your desktop web apps.
          </p>
        </div>

        <Button
          variant="primary"
          onClick={() => setAddModalOpen(true)}
          className="h-8 px-3 text-xs gap-1.5 justify-center"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Add {activeTab === 'plugins' ? 'Plugin (JS)' : 'Theme (CSS)'}</span>
        </Button>
      </div>

      <div className="mb-2">
        <Tabs
          label="Extension Categories"
          value={activeTab}
          onValueChange={(val: string) => setActiveTab(val as any)}
          items={[
            {
              value: 'plugins',
              label: `Plugins (JS) (${registry.plugins?.length || 0})`,
              content: renderContent,
            },
            {
              value: 'themes',
              label: `Themes (CSS) (${registry.themes?.length || 0})`,
              content: renderContent,
            },
          ]}
        />
      </div>

      <Modal
        open={addModalOpen}
        onOpenChange={setAddModalOpen}
        title={`New ${activeTab === 'plugins' ? 'JavaScript Plugin' : 'CSS Theme'}`}
        className="max-w-lg"
      >
        <div className="space-y-3.5 pt-1">
          <div>
            <label className="text-xs font-medium text-muted block mb-1">Extension Name</label>
            <TextInput
              label="Extension Name"
              hideLabel
              value={name}
              onChange={(val: string) => setName(val)}
              placeholder="e.g. Inverted Dark Reader, High Contrast"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-muted block mb-1">Description</label>
            <TextInput
              label="Description"
              hideLabel
              value={description}
              onChange={(val: string) => setDescription(val)}
              placeholder="Brief summary of injected functionality"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-xs font-medium text-muted block mb-1">Author</label>
              <TextInput
                label="Author"
                hideLabel
                value={author}
                onChange={(val: string) => setAuthor(val)}
                placeholder="Operator / Company"
              />
            </div>
            <div>
              <label className="text-xs font-medium text-muted block mb-1">Version</label>
              <TextInput
                label="Version"
                hideLabel
                value={version}
                onChange={(val: string) => setVersion(val)}
                placeholder="1.0.0"
              />
            </div>
          </div>

          <div>
            <label className="text-xs font-medium text-muted block mb-1.5">
              {activeTab === 'plugins' ? 'JavaScript Source Code' : 'CSS Stylesheet Rules'}
            </label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              rows={8}
              placeholder={
                activeTab === 'plugins'
                  ? '// JavaScript code to execute\nconsole.log("Custom script active");'
                  : '/* CSS rules to inject */\nbody { filter: contrast(1.1); }'
              }
              className="w-full font-mono text-xs p-3 rounded-lg bg-raised border border-border text-fg focus:outline-none focus:border-accent resize-none"
            />
          </div>

          <div className="pt-3 border-t border-border flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setAddModalOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleCreate}
              disabled={submitting || !name.trim() || !content.trim()}
            >
              {submitting ? 'Registering...' : 'Register Extension'}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
