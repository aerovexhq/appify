import React from 'react';
import { ExternalLink, ShieldCheck } from 'lucide-react';

interface FooterProps {
  onNavigate?: (tab: 'home' | 'docs', hashTarget?: string) => void;
}

export function Footer({ onNavigate }: FooterProps) {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="bg-[#08090a] border-t border-white/[0.08] text-slate-400 text-xs mt-auto">
      <div className="max-w-7xl mx-auto px-6 py-16">
        <div className="grid grid-cols-1 md:grid-cols-5 gap-10">
          <div className="md:col-span-2 space-y-4 pr-6">
            <div className="flex items-center gap-3">
              <img src="/favicon.png" alt="Appify" className="w-6 h-6 rounded-md object-contain border border-purple-500/30" />
              <span className="font-extrabold text-xl tracking-tight text-white font-sans">
                Appify<span className="text-purple-400">.</span>
              </span>
            </div>

            <p className="text-xs text-slate-400 leading-relaxed max-w-sm">
              Modern web-to-native desktop application bundler built on Tauri 2.0. Single-binary packaging, custom protocol schemes, sub-5MB native footprint, and Ed25519 differential auto-updates.
            </p>

            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-[#0f1012] border border-white/[0.08] text-[11px] font-mono text-slate-300">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
              <span>TAURI 2.0 ARCHITECTURE // &lt;5MB FOOTPRINT VERIFIED</span>
            </div>
          </div>

          <div className="space-y-3">
            <div className="text-xs font-mono font-bold text-white uppercase tracking-wider">
              Core Subsystems
            </div>
            <ul className="space-y-2 text-xs">
              <li>
                <a href="#subsystems" className="hover:text-white transition">
                  Single-Binary Bundler
                </a>
              </li>
              <li>
                <a href="#subsystems" className="hover:text-white transition">
                  Custom Protocol IPC
                </a>
              </li>
              <li>
                <a href="#subsystems" className="hover:text-white transition">
                  System Tray Window
                </a>
              </li>
              <li>
                <a href="#subsystems" className="hover:text-white transition">
                  Ed25519 Auto-Updater
                </a>
              </li>
              <li>
                <a href="#subsystems" className="hover:text-white transition">
                  Security Sandbox
                </a>
              </li>
            </ul>
          </div>

          <div className="space-y-3">
            <div className="text-xs font-mono font-bold text-white uppercase tracking-wider">
              Documentation
            </div>
            <ul className="space-y-2 text-xs">
              <li>
                <a href="#docs/getting_started" className="hover:text-white transition">
                  Getting Started
                </a>
              </li>
              <li>
                <a href="#docs/single_binary" className="hover:text-white transition">
                  Packaging Architecture
                </a>
              </li>
              <li>
                <a href="#docs/custom_protocol" className="hover:text-white transition">
                  IPC &amp; Protocols
                </a>
              </li>
              <li>
                <a href="#docs/ed25519_updater" className="hover:text-white transition">
                  Auto-Updates
                </a>
              </li>
              <li>
                <a href="#docs/cli_reference" className="hover:text-white transition">
                  Appify CLI Reference
                </a>
              </li>
            </ul>
          </div>

          <div className="space-y-3">
            <div className="text-xs font-mono font-bold text-white uppercase tracking-wider">
              Aerovex Ecosystem
            </div>
            <ul className="space-y-2 text-xs">
              <li>
                <a href="https://aerovex.net" target="_blank" rel="noopener noreferrer" className="hover:text-white transition flex items-center gap-1">
                  Aerovex Platform <ExternalLink className="w-3 h-3 text-slate-500" />
                </a>
              </li>
              <li>
                <a href="https://axiom.aerovex.net" target="_blank" rel="noopener noreferrer" className="hover:text-white transition flex items-center gap-1">
                  Axiom EDA <ExternalLink className="w-3 h-3 text-slate-500" />
                </a>
              </li>
              <li>
                <a href="https://phonon.aerovex.net" target="_blank" rel="noopener noreferrer" className="hover:text-white transition flex items-center gap-1">
                  Phonon SPICE <ExternalLink className="w-3 h-3 text-slate-500" />
                </a>
              </li>
              <li>
                <a href="https://atoma.aerovex.net" target="_blank" rel="noopener noreferrer" className="hover:text-white transition flex items-center gap-1">
                  Atoma Quantum CAD <ExternalLink className="w-3 h-3 text-slate-500" />
                </a>
              </li>
              <li>
                <a href="https://sonon.aerovex.net" target="_blank" rel="noopener noreferrer" className="hover:text-white transition flex items-center gap-1">
                  Sonon Acoustic DSP <ExternalLink className="w-3 h-3 text-slate-500" />
                </a>
              </li>
              <li>
                <a href="https://vexel.aerovex.net" target="_blank" rel="noopener noreferrer" className="hover:text-white transition flex items-center gap-1">
                  Vexel Systems Lang <ExternalLink className="w-3 h-3 text-slate-500" />
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="mt-14 pt-8 border-t border-white/[0.08] flex flex-col sm:flex-row items-center justify-between gap-4 text-slate-500 text-[11px] font-mono">
          <div>
            &copy; {currentYear} Aerovex. All rights reserved. Commercial Desktop Utilities.
          </div>
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1 text-slate-400">
              <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" /> TAURI 2.0
            </span>
            <span>ED25519 DELTAS</span>
            <span>&lt;5MB FOOTPRINT</span>
          </div>
        </div>
      </div>
    </footer>
  );
}
