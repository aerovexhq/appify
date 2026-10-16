import React, { useEffect, useState, useRef } from 'react';
import { createPortal } from 'react-dom';
import { X, Copy, Check, ExternalLink, Terminal, Cpu, ShieldCheck } from 'lucide-react';
import { Subsystem } from '../data/subsystems';

interface ArchitectureModalProps {
  subsystem: Subsystem | null;
  onClose: () => void;
  onNavigateToDocs?: (sectionId: string) => void;
}

export function ArchitectureModal({ subsystem, onClose, onNavigateToDocs }: ArchitectureModalProps) {
  const [copied, setCopied] = useState(false);
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!subsystem) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    window.addEventListener('keydown', handleKeyDown);

    const focusableElements = modalRef.current?.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusableElements && focusableElements.length > 0) {
      focusableElements[0].focus();
    }

    return () => {
      document.body.style.overflow = prevOverflow;
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [subsystem, onClose]);

  if (!subsystem || typeof document === 'undefined') return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(subsystem.architectureDetails.codeSnippet.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const modalContent = (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 overflow-y-auto bg-black/80 backdrop-blur-md animate-in fade-in duration-200"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={modalRef}
        className="relative w-full max-w-4xl bg-[#0f1012] border border-white/[0.08] rounded-2xl shadow-2xl overflow-hidden my-8 max-h-[90vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/[0.08] bg-[#15171a]">
          <div className="flex items-center gap-3">
            <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/30">
              {subsystem.tierLabel}
            </span>
            <span className="text-xs font-mono text-slate-400">
              {subsystem.category}
            </span>
          </div>

          <button
            type="button"
            onClick={onClose}
            aria-label="Close architecture modal"
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/[0.06] transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 sm:p-8 overflow-y-auto space-y-8 flex-1">
          <div>
            <h2 id="modal-title" className="text-2xl sm:text-3xl font-extrabold text-white tracking-tight mb-2 font-sans">
              {subsystem.name}
            </h2>
            <p className="text-sm text-slate-300 leading-relaxed">
              {subsystem.tagline}
            </p>
          </div>

          <div className="space-y-3">
            <h3 className="text-xs font-mono uppercase tracking-wider text-slate-400 font-semibold flex items-center gap-2">
              <Cpu className="w-3.5 h-3.5 text-purple-400" />
              <span>System Architecture &amp; Packaging Pipeline</span>
            </h3>
            <pre className="p-4 rounded-xl bg-[#08090a] border border-white/[0.08] text-purple-400 font-mono text-xs leading-relaxed overflow-x-auto">
              {subsystem.architectureDetails.pipelineDiagram}
            </pre>
          </div>

          <div className="space-y-3">
            <h3 className="text-xs font-mono uppercase tracking-wider text-slate-400 font-semibold flex items-center gap-2">
              <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
              <span>Architectural Trade-offs &amp; Desktop Invariants</span>
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {subsystem.architectureDetails.tradeoffs.map((item, idx) => (
                <div key={idx} className="p-4 rounded-xl bg-[#15171a] border border-white/[0.06] space-y-1.5">
                  <div className="text-xs font-mono font-bold text-white">
                    {item.choice}
                  </div>
                  <div className="text-xs text-slate-400 leading-relaxed">
                    {item.why}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-mono uppercase tracking-wider text-slate-400 font-semibold flex items-center gap-2">
                <Terminal className="w-3.5 h-3.5 text-purple-400" />
                <span>{subsystem.architectureDetails.codeSnippet.title}</span>
              </h3>
              <button
                type="button"
                onClick={handleCopy}
                className="flex items-center gap-1.5 text-xs font-mono text-slate-400 hover:text-white px-2.5 py-1 rounded bg-white/[0.04] border border-white/[0.08] transition"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copied ? 'Copied' : 'Copy Code'}</span>
              </button>
            </div>
            <pre className="p-4 rounded-xl bg-[#08090a] border border-white/[0.08] text-slate-200 font-mono text-xs leading-relaxed overflow-x-auto max-h-80">
              <code>{subsystem.architectureDetails.codeSnippet.code}</code>
            </pre>
          </div>
        </div>

        <div className="p-4 sm:p-6 border-t border-white/[0.08] bg-[#15171a] flex flex-wrap items-center justify-between gap-3">
          <div className="text-xs font-mono text-slate-400">
            METRIC: <span className="text-purple-400 font-bold">{subsystem.metric}</span> ({subsystem.metricDetail})
          </div>

          <div className="flex items-center gap-3">
            {onNavigateToDocs && (
              <button
                type="button"
                onClick={() => {
                  onClose();
                  onNavigateToDocs(subsystem.docSectionId);
                }}
                className="px-4 py-2 rounded-xl bg-purple-500 hover:bg-purple-400 text-white font-semibold text-xs transition flex items-center gap-1.5"
              >
                <span>Read Full Documentation</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            )}
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-xl bg-white/[0.06] hover:bg-white/[0.1] text-white text-xs font-medium transition"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );

  return createPortal(modalContent, document.body);
}
