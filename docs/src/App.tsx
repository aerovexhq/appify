import React, { useState, useEffect } from 'react';
import { BookOpen, Home as HomeIcon, ChevronDown } from 'lucide-react';
import DocsView from './DocsView';
import HomeView from './HomeView';
import { Footer } from './components/Footer';

interface SubdomainInfo {
  id: string;
  name: string;
  category: string;
  desc: string;
  url: string;
  isExternal: boolean;
  isActive?: boolean;
}

const ECOSYSTEM_SUBDOMAINS: SubdomainInfo[] = [
  {
    id: 'platform',
    name: 'Aerovex Platform',
    category: 'Flight Intelligence & Simulation',
    desc: 'Unified operations portal, 8.65M tick/s multi-world sim',
    url: 'https://aerovex.net',
    isExternal: true
  },
  {
    id: 'axiom',
    name: 'Axiom EDA',
    category: 'Silicon HDL & JIT',
    desc: 'In-RAM Verilog/SystemVerilog simulator & Cranelift JIT',
    url: 'https://axiom.aerovex.net',
    isExternal: true
  },
  {
    id: 'phonon',
    name: 'Phonon TCAD',
    category: 'Semiconductor TCAD',
    desc: 'Universal electro-thermal SPICE & 4.2K Cryo-CMOS simulator',
    url: 'https://phonon.aerovex.net',
    isExternal: true
  },
  {
    id: 'atoma',
    name: 'Atoma CAD',
    category: 'Quantum & Molecular CAD',
    desc: '3D atomistic studio, 14 Bravais lattices & Dirac cones',
    url: 'https://atoma.aerovex.net',
    isExternal: true
  },
  {
    id: 'sonon',
    name: 'Sonon DSP',
    category: 'Acoustic DSP',
    desc: 'Rotor noise notch filtering (>86 dB) & speech synthesis',
    url: 'https://sonon.aerovex.net',
    isExternal: true
  },
  {
    id: 'vexel',
    name: 'Vexel Language',
    category: 'Systems Compiler',
    desc: 'LLVM 18 SSA backend, non-lexical lifetimes & WASM DOM',
    url: 'https://vexel.aerovex.net',
    isExternal: true
  },
  {
    id: 'appify',
    name: 'Appify Bundler',
    category: 'Desktop Bundler',
    desc: 'Modern web-to-native desktop application bundler (Tauri 2.0)',
    url: 'https://appify.aerovex.net',
    isExternal: false,
    isActive: true
  }
];

export default function App() {
  const [activeTab, setActiveTabState] = useState<'home' | 'docs'>('home');
  const [ecosystemOpen, setEcosystemOpen] = useState(false);

  useEffect(() => {
    const handleClose = () => setEcosystemOpen(false);
    window.addEventListener('click', handleClose);
    return () => window.removeEventListener('click', handleClose);
  }, []);

  useEffect(() => {
    const handleHashChange = () => {
      const hash = window.location.hash.toLowerCase();
      if (hash.startsWith('#docs')) {
        setActiveTabState('docs');
      } else {
        setActiveTabState('home');
      }
    };

    handleHashChange();
    window.addEventListener('hashchange', handleHashChange);
    window.addEventListener('popstate', handleHashChange);
    return () => {
      window.removeEventListener('hashchange', handleHashChange);
      window.removeEventListener('popstate', handleHashChange);
    };
  }, []);

  const setActiveTab = (tab: 'home' | 'docs', hashTarget?: string) => {
    setActiveTabState(tab);
    if (hashTarget) {
      window.location.hash = hashTarget;
    } else if (tab === 'home') {
      window.location.hash = '#home';
    } else if (tab === 'docs') {
      if (!window.location.hash.startsWith('#docs')) {
        window.location.hash = '#docs';
      }
    }
  };

  return (
    <div className="min-h-screen flex flex-col bg-[#08090a] text-slate-100 font-sans selection:bg-purple-500/30 selection:text-white">
      {/* Fixed Blur Navigation Bar */}
      <nav className="sticky top-0 z-50 bg-[#08090a]/85 backdrop-blur-md border-b border-white/[0.08] px-6 py-3.5 flex items-center justify-between">
        <div 
          onClick={() => setActiveTab('home')}
          className="flex items-center gap-3 cursor-pointer select-none group"
        >
          <img
            src="/favicon.png"
            alt="Appify"
            className="w-8 h-8 rounded-lg object-contain border border-purple-500/30 shadow-md group-hover:scale-105 transition-transform"
          />
          <div className="flex items-center gap-2">
            <span className="font-extrabold text-xl tracking-tight text-white font-sans">
              Appify<span className="text-purple-400">.</span>
            </span>
            <span className="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-white/[0.06] text-slate-300 border border-white/[0.08]">
              TAURI 2.0 BUNDLER
            </span>
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse ml-0.5" title="Bundler Ready" />
          </div>
        </div>

        <div className="flex items-center gap-1.5 sm:gap-2 text-xs font-medium text-slate-300">
          <button
            type="button"
            onClick={() => setActiveTab('home')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg transition ${
              activeTab === 'home'
                ? 'bg-white/[0.08] text-white font-semibold'
                : 'hover:text-white hover:bg-white/[0.04]'
            }`}
          >
            <HomeIcon className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Home</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('docs', '#docs/why_appify')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg transition ${
              activeTab === 'docs'
                ? 'bg-white/[0.08] text-white font-semibold'
                : 'hover:text-white hover:bg-white/[0.04]'
            }`}
          >
            <BookOpen className="w-3.5 h-3.5" />
            <span>Docs</span>
          </button>

          {/* Ecosystem Switcher Dropdown */}
          <div className="relative">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                setEcosystemOpen(!ecosystemOpen);
              }}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg transition border border-white/[0.08] ${
                ecosystemOpen ? 'bg-[#15171a] text-white' : 'bg-[#0f1012] hover:bg-[#15171a] text-slate-300'
              }`}
            >
              <span className="w-2 h-2 rounded-full bg-[#5e6ad2]" />
              <span className="hidden md:inline">Aerovex Ecosystem</span>
              <ChevronDown className={`w-3.5 h-3.5 transition-transform ${ecosystemOpen ? 'rotate-180' : ''}`} />
            </button>

            {ecosystemOpen && (
              <div 
                className="absolute right-0 mt-2 w-80 bg-[#0f1012] border border-white/[0.08] rounded-xl shadow-2xl overflow-hidden py-1.5 z-50 animate-in fade-in zoom-in-95 duration-100"
                onClick={(e) => e.stopPropagation()}
              >
                <div className="px-3 py-2 border-b border-white/[0.06] text-[10px] font-mono uppercase tracking-wider text-slate-500">
                  Switch Aerovex Subsystem
                </div>
                <div className="divide-y divide-white/[0.04] max-h-96 overflow-y-auto">
                  {ECOSYSTEM_SUBDOMAINS.map((sub) => (
                    <a
                      key={sub.id}
                      href={sub.url}
                      className={`flex flex-col px-3.5 py-2.5 transition ${
                        sub.isActive
                          ? 'bg-purple-500/10 text-purple-400'
                          : 'hover:bg-white/[0.04] text-slate-300 hover:text-white'
                      }`}
                    >
                      <div className="flex items-center justify-between text-xs font-semibold">
                        <span>{sub.name}</span>
                        {sub.isActive && (
                          <span className="text-[9px] font-mono px-1.5 py-0.2 rounded bg-purple-500/20 text-purple-400">
                            Current
                          </span>
                        )}
                      </div>
                      <div className="text-[11px] text-slate-400 mt-0.5 line-clamp-1">
                        {sub.desc}
                      </div>
                    </a>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </nav>

      {/* Main Content Area */}
      {activeTab === 'home' ? (
        <HomeView onNavigate={setActiveTab} />
      ) : (
        <DocsView />
      )}

      {/* Footer */}
      <Footer onNavigate={setActiveTab} />
    </div>
  );
}
