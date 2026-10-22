import React, { useState } from 'react';
import { Download, Terminal, Copy, Check, ExternalLink, RefreshCw, Monitor, Apple, HardDrive } from 'lucide-react';
import { useReleases, Release } from '../hooks/useReleases';
import defaultReleasesData from '../data/releases.json';

interface DownloadSectionProps {
  repo?: string;
  installCommand?: string;
}

export function DownloadSection({
  repo = 'aerovexhq/appify',
  installCommand = 'curl -sSL https://raw.githubusercontent.com/aerovexhq/appify/main/install.sh | bash',
}: DownloadSectionProps) {
  const { latestRelease, loading, error } = useReleases(repo, defaultReleasesData as Release[]);
  const [copied, setCopied] = useState(false);
  const [platformTab, setPlatformTab] = useState<'all' | 'linux' | 'macos' | 'windows'>('all');

  const handleCopy = () => {
    navigator.clipboard.writeText(installCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const formatSize = (bytes: number): string => {
    if (!bytes || bytes <= 0) return 'Unknown size';
    const mb = bytes / (1024 * 1024);
    if (mb < 1) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${mb.toFixed(1)} MB`;
  };

  const categorizeAsset = (name: string): { os: 'linux' | 'macos' | 'windows' | 'other'; label: string } => {
    const n = name.toLowerCase();
    if (n.endsWith('.appimage')) return { os: 'linux', label: 'Linux AppImage (Universal)' };
    if (n.endsWith('.deb')) return { os: 'linux', label: 'Debian / Ubuntu Package' };
    if (n.endsWith('.rpm')) return { os: 'linux', label: 'Fedora / RHEL Package' };
    if (n.includes('aarch64.dmg') || n.includes('arm64.dmg')) return { os: 'macos', label: 'macOS Apple Silicon (.dmg)' };
    if (n.includes('x64.dmg')) return { os: 'macos', label: 'macOS Intel (.dmg)' };
    if (n.endsWith('.dmg')) return { os: 'macos', label: 'macOS Bundle (.dmg)' };
    if (n.endsWith('.exe')) return { os: 'windows', label: 'Windows Setup (.exe)' };
    if (n.endsWith('.msi')) return { os: 'windows', label: 'Windows MSI Installer' };
    if (n.endsWith('.tar.gz')) return { os: 'linux', label: 'Linux Portable Archive' };
    return { os: 'other', label: 'Binary Archive' };
  };

  const allAssets = latestRelease?.assets || [];
  const filteredAssets = allAssets.filter((asset) => {
    if (platformTab === 'all') return true;
    const cat = categorizeAsset(asset.name);
    return cat.os === platformTab;
  });

  return (
    <section id="downloads" className="py-20 px-6 max-w-7xl mx-auto w-full z-10 relative">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-12">
        <div>
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/[0.04] border border-white/[0.08] text-xs font-mono text-purple-400 mb-3">
            <Download className="w-3.5 h-3.5" />
            <span>STANDALONE DISTRIBUTIONS // GITHUB RELEASES</span>
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-white tracking-tight font-sans">
            Download Appify Desktop
          </h2>
          <p className="text-sm text-slate-400 mt-2 max-w-2xl leading-relaxed">
            Multi-platform native executables packaged with zero background runtime bloat. Dynamic releases fetched directly from GitHub with verified compile-time fallbacks.
          </p>
        </div>

        {latestRelease && (
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 px-3.5 py-1.5 rounded-xl bg-[#0f1012] border border-white/[0.08] text-xs font-mono text-slate-300">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              <span>Version: <strong className="text-white">{latestRelease.tag_name}</strong></span>
              {loading && <RefreshCw className="w-3 h-3 animate-spin text-purple-400 ml-1" />}
            </div>
            <a
              href={latestRelease.html_url}
              target="_blank"
              rel="noopener noreferrer"
              className="p-2 rounded-xl bg-[#0f1012] border border-white/[0.08] hover:border-white/[0.16] text-slate-400 hover:text-white transition"
              title="View on GitHub"
            >
              <ExternalLink className="w-4 h-4" />
            </a>
          </div>
        )}
      </div>

      {/* Terminal Quick-Install Strip */}
      <div className="mb-10 p-5 rounded-2xl bg-[#0f1012] border border-white/[0.08] relative overflow-hidden">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-white/[0.05] border border-white/[0.08] flex items-center justify-center text-purple-400">
              <Terminal className="w-4 h-4" />
            </div>
            <div>
              <div className="text-xs font-mono font-medium text-slate-300">Quick Install (Linux / macOS)</div>
              <div className="text-[11px] font-mono text-slate-500">Universal curl bootstrapper with automated architecture detection</div>
            </div>
          </div>

          <div className="flex items-center gap-2 w-full md:w-auto bg-[#08090a] px-3.5 py-2.5 rounded-xl border border-white/[0.06] font-mono text-xs text-purple-300 overflow-x-auto justify-between">
            <code className="select-all whitespace-nowrap">{installCommand}</code>
            <button
              type="button"
              onClick={handleCopy}
              className="p-1.5 rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-slate-300 hover:text-white transition flex-shrink-0 ml-3"
              title="Copy install command"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
            </button>
          </div>
        </div>
      </div>

      {/* Platform Filter Tabs */}
      <div className="flex items-center justify-between flex-wrap gap-4 mb-6">
        <div className="flex items-center gap-1.5 bg-[#0f1012] p-1.5 rounded-xl border border-white/[0.08]">
          <button
            type="button"
            onClick={() => setPlatformTab('all')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition ${
              platformTab === 'all'
                ? 'bg-purple-600 text-white font-semibold shadow-sm'
                : 'text-slate-400 hover:text-white hover:bg-white/[0.04]'
            }`}
          >
            <span>All Packages</span>
            <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-white/[0.06] text-slate-400">
              {allAssets.length}
            </span>
          </button>

          <button
            type="button"
            onClick={() => setPlatformTab('linux')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition ${
              platformTab === 'linux'
                ? 'bg-purple-600 text-white font-semibold shadow-sm'
                : 'text-slate-400 hover:text-white hover:bg-white/[0.04]'
            }`}
          >
            <HardDrive className="w-3.5 h-3.5" />
            <span>Linux</span>
          </button>

          <button
            type="button"
            onClick={() => setPlatformTab('macos')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition ${
              platformTab === 'macos'
                ? 'bg-purple-600 text-white font-semibold shadow-sm'
                : 'text-slate-400 hover:text-white hover:bg-white/[0.04]'
            }`}
          >
            <Apple className="w-3.5 h-3.5" />
            <span>macOS</span>
          </button>

          <button
            type="button"
            onClick={() => setPlatformTab('windows')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition ${
              platformTab === 'windows'
                ? 'bg-purple-600 text-white font-semibold shadow-sm'
                : 'text-slate-400 hover:text-white hover:bg-white/[0.04]'
            }`}
          >
            <Monitor className="w-3.5 h-3.5" />
            <span>Windows</span>
          </button>
        </div>

        {error && (
          <div className="text-[11px] font-mono text-amber-400/80">
            Runtime sync Notice: Using compile-time release snapshot
          </div>
        )}
      </div>

      {/* Asset Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredAssets.map((asset) => {
          const { label } = categorizeAsset(asset.name);
          return (
            <div
              key={asset.name}
              className="p-5 rounded-2xl bg-[#0f1012] border border-white/[0.08] hover:border-purple-500/30 transition-all flex flex-col justify-between group"
            >
              <div>
                <div className="flex items-center justify-between mb-3">
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-purple-500/10 text-purple-300 border border-purple-500/20">
                    {label}
                  </span>
                  <span className="text-xs font-mono text-slate-500">{formatSize(asset.size)}</span>
                </div>
                <h4 className="text-sm font-semibold text-white tracking-tight font-mono break-all mb-1 group-hover:text-purple-300 transition-colors">
                  {asset.name}
                </h4>
              </div>

              <div className="mt-5 pt-4 border-t border-white/[0.06] flex items-center justify-between">
                <span className="text-[11px] font-mono text-slate-500">
                  {asset.download_count > 0 ? `${asset.download_count} downloads` : 'Direct build'}
                </span>
                <a
                  href={asset.browser_download_url}
                  download
                  className="px-4 py-2 rounded-xl bg-purple-600/20 hover:bg-purple-600 border border-purple-500/30 text-purple-200 hover:text-white text-xs font-semibold font-mono flex items-center gap-1.5 transition-all shadow-md"
                >
                  <Download className="w-3.5 h-3.5" />
                  <span>Download</span>
                </a>
              </div>
            </div>
          );
        })}
      </div>

      {filteredAssets.length === 0 && (
        <div className="text-center py-12 text-slate-500 font-mono text-xs">
          No release assets found for this platform filter.
        </div>
      )}
    </section>
  );
}
