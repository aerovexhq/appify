import React from 'react';
import { Layers, ChevronRight } from 'lucide-react';
import { Subsystem } from '../data/subsystems';
import { useSpotlight } from '../hooks/useSpotlight';

interface SubsystemCardProps {
  subsystem: Subsystem;
  onInspect: (subsystem: Subsystem) => void;
  onNavigateToDocs: (targetId: string) => void;
}

export function SubsystemCard({ subsystem, onInspect, onNavigateToDocs }: SubsystemCardProps) {
  const { ref, onMouseMove, onMouseLeave } = useSpotlight<HTMLDivElement>();

  const getStatusToneClass = (tone: string) => {
    switch (tone) {
      case 'emerald':
        return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30';
      case 'cyan':
        return 'text-cyan-400 bg-cyan-500/10 border-cyan-500/30';
      case 'amber':
        return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
      case 'purple':
        return 'text-purple-400 bg-purple-500/10 border-purple-500/30';
      default:
        return 'text-purple-400 bg-purple-500/10 border-purple-500/30';
    }
  };

  return (
    <div
      ref={ref}
      onMouseMove={onMouseMove}
      onMouseLeave={onMouseLeave}
      className="nx-spotlight-card p-6 md:p-7 flex flex-col justify-between group"
    >
      <div>
        <div className="flex items-center justify-between gap-3 mb-4">
          <div className="flex items-center gap-2">
            <span className="text-[10px] font-mono uppercase tracking-wider px-2 py-0.5 rounded bg-white/[0.06] text-slate-300 border border-white/[0.08]">
              {subsystem.tierLabel}
            </span>
            <span className="text-[10px] font-mono text-slate-500">
              {subsystem.category}
            </span>
          </div>

          <div className={`flex items-center gap-1.5 px-2.5 py-0.5 rounded-full border text-[10px] font-mono uppercase tracking-wider ${getStatusToneClass(subsystem.statusTone)}`}>
            <span className="w-1.5 h-1.5 rounded-full bg-current animate-pulse"></span>
            <span>{subsystem.status}</span>
          </div>
        </div>

        <h3 className="text-xl md:text-2xl font-bold tracking-tight text-white mb-2 font-sans group-hover:text-white transition">
          {subsystem.name}
        </h3>
        <p className="text-xs text-slate-400 leading-relaxed mb-5">
          {subsystem.tagline}
        </p>

        <div className="bg-[#08090a] border border-white/[0.08] rounded-xl p-4 mb-5">
          <div className="text-2xl font-mono font-bold text-purple-400 tracking-tight">
            {subsystem.metric}
          </div>
          <div className="text-[11px] font-mono text-slate-400 mt-0.5">
            {subsystem.metricDetail}
          </div>
        </div>

        <p className="text-xs text-slate-300 leading-relaxed mb-5">
          {subsystem.description}
        </p>

        <ul className="space-y-2 mb-6">
          {subsystem.highlights.map((item, idx) => (
            <li key={idx} className="flex items-start gap-2 text-xs text-slate-400">
              <span className="w-1.5 h-1.5 rounded-full bg-purple-400 mt-1.5 shrink-0"></span>
              <span className="leading-relaxed">{item}</span>
            </li>
          ))}
        </ul>

        <div className="flex flex-wrap items-center gap-1.5 mb-6">
          {subsystem.tags.map((tag, idx) => (
            <span
              key={idx}
              className="px-2 py-0.5 rounded bg-white/[0.04] text-[10px] font-mono text-slate-400 border border-white/[0.06]"
            >
              {tag}
            </span>
          ))}
        </div>
      </div>

      <div className="pt-4 border-t border-white/[0.08] flex items-center justify-between gap-3">
        <button
          type="button"
          onClick={() => onInspect(subsystem)}
          className="text-xs font-mono font-semibold text-white hover:text-purple-400 flex items-center gap-1.5 transition"
        >
          <Layers className="w-3.5 h-3.5" />
          <span>Inspect Architecture</span>
        </button>

        <button
          type="button"
          onClick={() => onNavigateToDocs(subsystem.docSectionId)}
          className="text-xs font-mono text-slate-400 hover:text-white flex items-center gap-1 transition"
        >
          <span>Docs</span>
          <ChevronRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
