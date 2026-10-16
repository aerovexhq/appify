import React, { useState, useEffect } from 'react';
import { 
  Search, ChevronRight, ChevronDown, Copy, Check, ArrowLeft, ArrowRight, Cpu, ShieldCheck
} from 'lucide-react';
import { APPIFY_DOC_TREE, DocSection } from './docsData';

interface DocsViewProps {
  onLoadPreset?: (code: string) => void;
}

interface InlineCodeBlockProps {
  section: DocSection;
}

function InlineCodeBlock({ section }: InlineCodeBlockProps) {
  const [codeTab, setCodeTab] = useState<'config' | 'rust' | 'ts'>('config');
  const [copied, setCopied] = useState<boolean>(false);

  const getActiveCode = () => {
    if (codeTab === 'rust' && section.content.rustCode) return section.content.rustCode;
    if (codeTab === 'ts' && section.content.tsCode) return section.content.tsCode;
    return section.content.configJson || '';
  };

  const handleCopyCode = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const activeCode = getActiveCode();
  if (!activeCode) return null;

  return (
    <div className="bg-[#0f1012] rounded-xl border border-white/[0.08] overflow-hidden mb-6 shadow-xl">
      <div className="bg-[#15171a] px-4 py-2 border-b border-white/[0.08] flex items-center justify-between">
        <div className="flex items-center gap-1.5">
          {section.content.configJson && (
            <button
              onClick={() => setCodeTab('config')}
              className={`px-3 py-1 rounded-lg text-xs font-semibold font-mono transition ${
                codeTab === 'config' ? 'bg-purple-600 text-white shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              appify.config.json
            </button>
          )}
          {section.content.rustCode && (
            <button
              onClick={() => setCodeTab('rust')}
              className={`px-3 py-1 rounded-lg text-xs font-semibold font-mono transition ${
                codeTab === 'rust' ? 'bg-emerald-600 text-white shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              Rust IPC Handler
            </button>
          )}
          {section.content.tsCode && (
            <button
              onClick={() => setCodeTab('ts')}
              className={`px-3 py-1 rounded-lg text-xs font-semibold font-mono transition ${
                codeTab === 'ts' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-white'
              }`}
            >
              TypeScript Client
            </button>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => handleCopyCode(activeCode)}
            className="flex items-center gap-1 text-[11px] font-mono text-slate-400 hover:text-white px-2.5 py-1 rounded transition hover:bg-white/[0.05]"
            title="Copy snippet"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
            <span>{copied ? 'Copied' : 'Copy'}</span>
          </button>
        </div>
      </div>

      <div className="p-4 bg-[#08090a] font-mono text-xs text-slate-200 overflow-x-auto">
        <pre className="leading-relaxed">
          <code>{activeCode}</code>
        </pre>
      </div>
    </div>
  );
}

function ArchitectureSpecTable({ specs }: { specs?: { param: string; val: string; note: string }[] }) {
  if (!specs || specs.length === 0) return null;

  return (
    <div className="bg-[#0f1012] rounded-xl border border-white/[0.08] overflow-hidden mb-6">
      <div className="px-4 py-2.5 bg-[#15171a] border-b border-white/[0.08] flex items-center justify-between">
        <span className="text-[11px] font-mono font-semibold text-slate-300 uppercase tracking-wider flex items-center gap-2">
          <Cpu className="w-3.5 h-3.5 text-purple-400" /> Desktop Runtime Specifications
        </span>
        <span className="text-[10px] font-mono text-slate-500">TAURI 2.0 CORE</span>
      </div>
      <div className="divide-y divide-white/[0.04]">
        {specs.map((s, idx) => (
          <div key={idx} className="p-3 sm:px-4 flex flex-col sm:flex-row sm:items-center justify-between gap-1 text-xs">
            <span className="font-mono text-slate-300 font-medium">{s.param}</span>
            <div className="flex items-center gap-3">
              <span className="font-mono font-bold text-purple-400">{s.val}</span>
              <span className="text-[11px] text-slate-500 font-sans hidden md:inline">({s.note})</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function CalloutBox({ callout }: { callout?: { type: 'note' | 'tip' | 'warning' | 'spec'; title: string; text: string } }) {
  if (!callout) return null;

  const getBorderAndBg = () => {
    switch (callout.type) {
      case 'tip':
        return 'border-emerald-500/30 bg-emerald-500/[0.05] text-emerald-300';
      case 'warning':
        return 'border-amber-500/30 bg-amber-500/[0.05] text-amber-300';
      case 'spec':
        return 'border-purple-500/30 bg-purple-500/[0.05] text-purple-300';
      default:
        return 'border-white/[0.12] bg-white/[0.03] text-slate-300';
    }
  };

  return (
    <div className={`p-4 rounded-xl border ${getBorderAndBg()} mb-6`}>
      <div className="flex items-center gap-2 text-xs font-mono font-bold uppercase tracking-wider mb-1.5">
        <ShieldCheck className="w-4 h-4 shrink-0" />
        <span>{callout.title}</span>
      </div>
      <p className="text-xs text-slate-300 leading-relaxed font-sans">
        {callout.text}
      </p>
    </div>
  );
}

export default function DocsView({ onLoadPreset }: DocsViewProps) {
  const [activeCategory, setActiveCategory] = useState<string>(APPIFY_DOC_TREE[0].id);
  const [activeSectionId, setActiveSectionId] = useState<string>(APPIFY_DOC_TREE[0].sections[0].id);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({
    [APPIFY_DOC_TREE[0].id]: true,
    [APPIFY_DOC_TREE[1].id]: true,
    [APPIFY_DOC_TREE[2].id]: true,
    [APPIFY_DOC_TREE[3].id]: true,
  });

  useEffect(() => {
    const handleHash = () => {
      const hash = window.location.hash;
      if (hash.startsWith('#docs/')) {
        const targetId = hash.replace('#docs/', '');
        for (const cat of APPIFY_DOC_TREE) {
          const found = cat.sections.find((s) => s.id === targetId);
          if (found) {
            setActiveCategory(cat.id);
            setActiveSectionId(found.id);
            setExpandedCategories((prev) => ({ ...prev, [cat.id]: true }));
            return;
          }
        }
      }
    };

    handleHash();
    window.addEventListener('hashchange', handleHash);
    return () => window.removeEventListener('hashchange', handleHash);
  }, []);

  const toggleCategory = (catId: string) => {
    setExpandedCategories((prev) => ({
      ...prev,
      [catId]: !prev[catId],
    }));
  };

  const allSections: { cat: string; section: DocSection }[] = [];
  APPIFY_DOC_TREE.forEach((cat) => {
    cat.sections.forEach((sec) => {
      allSections.push({ cat: cat.id, section: sec });
    });
  });

  const currentIndex = allSections.findIndex((item) => item.section.id === activeSectionId);
  const prevItem = currentIndex > 0 ? allSections[currentIndex - 1] : null;
  const nextItem = currentIndex < allSections.length - 1 ? allSections[currentIndex + 1] : null;

  const currentCat = APPIFY_DOC_TREE.find((c) => c.id === activeCategory);
  const currentSection = currentCat?.sections.find((s) => s.id === activeSectionId) || APPIFY_DOC_TREE[0].sections[0];

  const filteredCategories = APPIFY_DOC_TREE.map((cat) => {
    const matchingSections = cat.sections.filter(
      (sec) =>
        sec.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        sec.content.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        sec.badge.toLowerCase().includes(searchQuery.toLowerCase())
    );
    return { ...cat, sections: matchingSections };
  }).filter((cat) => cat.sections.length > 0);

  const selectSection = (catId: string, secId: string) => {
    setActiveCategory(catId);
    setActiveSectionId(secId);
    window.location.hash = `#docs/${secId}`;
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  return (
    <div className="flex-1 flex max-w-7xl mx-auto w-full px-4 sm:px-6 py-6 md:py-8 gap-8">
      {/* Sidebar Navigation */}
      <aside className="w-64 sm:w-72 shrink-0 hidden md:block">
        <div className="sticky top-20 space-y-4">
          {/* Search Box */}
          <div className="relative">
            <Search className="w-3.5 h-3.5 absolute left-3 top-3 text-slate-400" />
            <input
              type="text"
              placeholder="Search documentation..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-[#0f1012] border border-white/[0.08] focus:border-purple-500/50 rounded-xl pl-9 pr-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none transition font-sans"
            />
          </div>

          {/* Navigation Tree */}
          <div className="space-y-1 max-h-[calc(100vh-160px)] overflow-y-auto pr-1">
            {filteredCategories.map((cat) => (
              <div key={cat.id} className="mb-3">
                <button
                  type="button"
                  onClick={() => toggleCategory(cat.id)}
                  className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs font-mono font-bold text-slate-300 hover:text-white transition rounded-lg hover:bg-white/[0.03]"
                >
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] text-purple-400 bg-purple-500/10 px-1.5 py-0.2 rounded border border-purple-500/20">
                      {cat.badge}
                    </span>
                    <span>{cat.title}</span>
                  </div>
                  {expandedCategories[cat.id] ? (
                    <ChevronDown className="w-3.5 h-3.5 text-slate-400" />
                  ) : (
                    <ChevronRight className="w-3.5 h-3.5 text-slate-400" />
                  )}
                </button>

                {expandedCategories[cat.id] && (
                  <div className="mt-1 ml-3 pl-2.5 border-l border-white/[0.08] space-y-0.5">
                    {cat.sections.map((sec) => {
                      const isActive = activeSectionId === sec.id;
                      return (
                        <button
                          key={sec.id}
                          type="button"
                          onClick={() => selectSection(cat.id, sec.id)}
                          className={`w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-sans transition flex items-center justify-between group ${
                            isActive
                              ? 'bg-purple-500/10 text-purple-400 font-semibold border-l-2 border-purple-400 -ml-[11px] pl-[17px]'
                              : 'text-slate-400 hover:text-white hover:bg-white/[0.03]'
                          }`}
                        >
                          <span className="truncate">{sec.title}</span>
                          <span className="text-[9px] font-mono text-slate-500 uppercase opacity-0 group-hover:opacity-100 transition">
                            {sec.badge}
                          </span>
                        </button>
                      );
                    })}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </aside>

      {/* Main Documentation Article */}
      <main className="flex-1 min-w-0 max-w-3xl">
        <div className="mb-6 flex items-center gap-2 text-xs font-mono text-slate-400">
          <span>Docs</span>
          <ChevronRight className="w-3 h-3 text-slate-600" />
          <span>{currentCat?.title}</span>
          <ChevronRight className="w-3 h-3 text-slate-600" />
          <span className="text-purple-400 font-bold">{currentSection.title}</span>
        </div>

        <div className="mb-8">
          <div className="flex items-center gap-2 mb-3">
            <span className="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/30">
              {currentSection.content.badge}
            </span>
          </div>

          <h1 className="text-3xl sm:text-4xl font-extrabold text-white tracking-tight mb-4 font-sans">
            {currentSection.content.title}
          </h1>

          <p className="text-sm text-slate-300 leading-relaxed font-sans">
            {currentSection.content.description}
          </p>
        </div>

        {/* Callout Box */}
        <CalloutBox callout={currentSection.content.callout} />

        {/* Specifications Table */}
        <ArchitectureSpecTable specs={currentSection.content.specs} />

        {/* Code Snippets */}
        <InlineCodeBlock section={currentSection} />

        {/* Steps or CLI instructions if available */}
        {currentSection.content.steps && (
          <div className="space-y-4 mb-8">
            <div className="text-xs font-mono font-bold text-white uppercase tracking-wider">
              Execution Sequence
            </div>
            {currentSection.content.steps.map((st, idx) => (
              <div key={idx} className="p-4 rounded-xl bg-[#0f1012] border border-white/[0.08]">
                <div className="flex items-center gap-2 text-xs font-bold text-white mb-1">
                  <span className="w-5 h-5 rounded-full bg-purple-500/20 text-purple-400 flex items-center justify-center font-mono text-[10px]">
                    {idx + 1}
                  </span>
                  <span>{st.title}</span>
                </div>
                <p className="text-xs text-slate-400 mb-2">{st.desc}</p>
                {st.cmd && (
                  <pre className="p-2.5 rounded-lg bg-[#08090a] border border-white/[0.06] text-purple-400 font-mono text-[11px] overflow-x-auto">
                    <code>{st.cmd}</code>
                  </pre>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Previous / Next Navigation Footer */}
        <div className="mt-12 pt-6 border-t border-white/[0.08] flex items-center justify-between gap-4">
          {prevItem ? (
            <button
              type="button"
              onClick={() => selectSection(prevItem.cat, prevItem.section.id)}
              className="flex items-center gap-2 text-xs font-mono text-slate-400 hover:text-white transition group text-left"
            >
              <ArrowLeft className="w-3.5 h-3.5 group-hover:-translate-x-1 transition" />
              <div>
                <div className="text-[10px] text-slate-500 uppercase">Previous</div>
                <div className="text-white font-medium">{prevItem.section.title}</div>
              </div>
            </button>
          ) : <div />}

          {nextItem && (
            <button
              type="button"
              onClick={() => selectSection(nextItem.cat, nextItem.section.id)}
              className="flex items-center gap-2 text-xs font-mono text-slate-400 hover:text-white transition group text-right ml-auto"
            >
              <div>
                <div className="text-[10px] text-slate-500 uppercase">Next</div>
                <div className="text-white font-medium">{nextItem.section.title}</div>
              </div>
              <ArrowRight className="w-3.5 h-3.5 group-hover:translate-x-1 transition" />
            </button>
          )}
        </div>
      </main>
    </div>
  );
}
