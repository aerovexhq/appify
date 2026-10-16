import React from 'react';
import { BarChart3, Zap } from 'lucide-react';

interface MatrixRow {
  metric: string;
  category: string;
  appify: { text: string; highlight?: boolean };
  electron: { text: string };
  tauriStd: { text: string };
  flutter: { text: string };
}

const COMPARISON_ROWS: MatrixRow[] = [
  {
    metric: 'Standalone Distributable Binary Size',
    category: 'Bundle Size',
    appify: { text: '<4.8 MB (Zero Chromium Bundling)', highlight: true },
    electron: { text: '145 - 220 MB (Chromium + Node)' },
    tauriStd: { text: '12 - 18 MB' },
    flutter: { text: '45 - 65 MB' }
  },
  {
    metric: 'Idle Process Memory Footprint',
    category: 'RAM Overhead',
    appify: { text: '<24 MB (Native OS WebKit/WebView2)', highlight: true },
    electron: { text: '180 - 350 MB (V8 + Blink)' },
    tauriStd: { text: '35 - 50 MB' },
    flutter: { text: '85 - 120 MB' }
  },
  {
    metric: 'Native IPC Communication Round-Trip',
    category: 'IPC Throughput',
    appify: { text: '<0.12 ms (Direct Binary Memory)', highlight: true },
    electron: { text: '1.45 ms (Node IPC channel)' },
    tauriStd: { text: '0.45 ms (JSON serialization)' },
    flutter: { text: '0.85 ms (Platform channels)' }
  },
  {
    metric: 'Cryptographic Security & Sandbox',
    category: 'Security Invariant',
    appify: { text: 'Ed25519 Delta + Strict ACL CSP', highlight: true },
    electron: { text: 'Vulnerable if nodeIntegration used' },
    tauriStd: { text: 'Tauri standard ACL' },
    flutter: { text: 'Standard OS sandboxing' }
  },
  {
    metric: 'Application Cold Start Launch Time',
    category: 'Startup Latency',
    appify: { text: '<180 ms (Instant Native Window)', highlight: true },
    electron: { text: '1,800 - 3,200 ms' },
    tauriStd: { text: '420 ms' },
    flutter: { text: '650 ms' }
  },
  {
    metric: 'Differential Auto-Updater Overhead',
    category: 'Update Bandwidth',
    appify: { text: '<200 KB (Binary Byte Delta)', highlight: true },
    electron: { text: '80 - 150 MB (Full package download)' },
    tauriStd: { text: 'Full binary download (~15MB)' },
    flutter: { text: 'Manual package distribution' }
  }
];

export function BenchmarkMatrix() {
  return (
    <section className="py-20 px-6 max-w-7xl mx-auto w-full z-10 relative">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-12">
        <div>
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/[0.04] border border-white/[0.08] text-xs font-mono text-purple-400 mb-3">
            <BarChart3 className="w-3.5 h-3.5" />
            <span>EMPIRICAL BENCHMARKS // DESKTOP BUNDLER PERFORMANCE</span>
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-white tracking-tight font-sans">
            Appify vs Classical Desktop Runtimes
          </h2>
          <p className="text-sm text-slate-400 mt-2 max-w-2xl leading-relaxed">
            Direct comparison across standalone binary size, idle RAM consumption, cold start latency, and zero-copy IPC bridge throughput.
          </p>
        </div>

        <div className="flex items-center gap-2 font-mono text-xs text-slate-400 bg-[#0f1012] p-2 rounded-xl border border-white/[0.08]">
          <span className="w-2 h-2 rounded-full bg-purple-400 animate-pulse"></span>
          <span>BENCHMARK SUITE: LINUX X86_64 STANDALONE LAUNCH</span>
        </div>
      </div>

      <div className="bg-[#0f1012] border border-white/[0.08] rounded-2xl overflow-hidden shadow-2xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse text-xs">
            <thead>
              <tr className="border-b border-white/[0.08] bg-[#15171a] font-mono text-[11px] text-slate-300">
                <th className="py-4 px-6 font-bold uppercase tracking-wider">Evaluation Dimension</th>
                <th className="py-4 px-6 font-bold text-purple-400 uppercase tracking-wider bg-purple-500/[0.06] border-x border-purple-500/20">
                  Appify Tauri 2.0
                </th>
                <th className="py-4 px-6 font-semibold uppercase tracking-wider text-slate-400">Electron 30</th>
                <th className="py-4 px-6 font-semibold uppercase tracking-wider text-slate-400">Tauri 1.x Default</th>
                <th className="py-4 px-6 font-semibold uppercase tracking-wider text-slate-400">Flutter 3.x</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/[0.04] font-mono">
              {COMPARISON_ROWS.map((row, idx) => (
                <tr key={idx} className="hover:bg-white/[0.02] transition">
                  <td className="py-4 px-6 text-slate-300 font-sans font-medium">
                    <div className="text-sm font-semibold text-white">{row.metric}</div>
                    <div className="text-xs text-slate-500 font-mono mt-0.5">{row.category}</div>
                  </td>

                  <td className="py-4 px-6 text-white font-bold bg-purple-500/[0.04] border-x border-purple-500/10">
                    <div className="flex items-center gap-2 text-purple-400">
                      <Zap className="w-3.5 h-3.5 fill-current shrink-0" />
                      <span>{row.appify.text}</span>
                    </div>
                  </td>

                  <td className="py-4 px-6 text-slate-400">{row.electron.text}</td>
                  <td className="py-4 px-6 text-slate-400">{row.tauriStd.text}</td>
                  <td className="py-4 px-6 text-slate-400">{row.flutter.text}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
