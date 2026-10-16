<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { SUBSYSTEMS, BENCHMARK_COMPARISONS, type Subsystem } from '../data/subsystems'

// Subsystem Modal state
const selectedSubsystem = ref<Subsystem | null>(null)
const copiedSnippet = ref(false)

// Domain filter tabs for Bento Grid
type FilterKey = 'all' | 'runtime' | 'bundling' | 'ipc'
const activeFilter = ref<FilterKey>('all')

const filteredSubsystems = computed(() => {
  if (activeFilter.value === 'runtime') {
    return SUBSYSTEMS.filter(s => s.id === 'tauri-shell' || s.id === 'memfd-execution')
  }
  if (activeFilter.value === 'bundling') {
    return SUBSYSTEMS.filter(s => s.id === 'sub5mb-bundle' || s.id === 'zero-config')
  }
  if (activeFilter.value === 'ipc') {
    return SUBSYSTEMS.filter(s => s.id === 'system-tray-ipc')
  }
  return SUBSYSTEMS
})

// Ecosystem dropdown state
const ecosystemOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

function toggleEcosystem() {
  ecosystemOpen.value = !ecosystemOpen.value
}

function closeEcosystem() {
  ecosystemOpen.value = false
}

// Mouse-tracking spotlight for Bento Grid cards
const cardRefs = ref<HTMLElement[]>([])

function setCardRef(el: any, index: number) {
  if (el) cardRefs.value[index] = el
}

function handleMouseMove(e: MouseEvent, index: number) {
  const card = cardRefs.value[index]
  if (!card) return
  const rect = card.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top
  card.style.setProperty('--mouse-x', `${x}px`)
  card.style.setProperty('--mouse-y', `${y}px`)
  card.style.setProperty('--spotlight-opacity', '1')
}

function handleMouseLeave(index: number) {
  const card = cardRefs.value[index]
  if (!card) return
  card.style.setProperty('--spotlight-opacity', '0')
}

// Architecture Modal Open / Close
function openArchitecture(subsystem: Subsystem) {
  selectedSubsystem.value = subsystem
  copiedSnippet.value = false
  if (typeof document !== 'undefined') {
    document.body.style.overflow = 'hidden'
  }
}

function closeArchitecture() {
  selectedSubsystem.value = null
  copiedSnippet.value = false
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }
}

function copySnippetText(code: string) {
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    navigator.clipboard.writeText(code)
    copiedSnippet.value = true
    setTimeout(() => {
      copiedSnippet.value = false
    }, 2000)
  }
}

// Keydown listener for Modal ESC & Dropdown ESC
function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (selectedSubsystem.value) {
      closeArchitecture()
    }
    if (ecosystemOpen.value) {
      closeEcosystem()
    }
  }
}

function handleClickOutside(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    ecosystemOpen.value = false
  }
}

// Interactive Code Tabs in Pipeline Showcase
type PipelineTabKey = 'cli' | 'manifest' | 'tray' | 'memfd'
const activePipelineTab = ref<PipelineTabKey>('cli')
const pipelineCopied = ref(false)

const pipelineSnippets: Record<PipelineTabKey, { filename: string; language: string; code: string }> = {
  cli: {
    filename: 'cli_pack.sh',
    language: 'bash',
    code: `# Create and package Linear desktop app in one command
appify create linear \\
  --url "https://linear.app" \\
  --title "Linear" \\
  --icon "./icons/linear.png" \\
  --tray \\
  --autostart \\
  --isolated-session`,
  },
  manifest: {
    filename: 'appify.config.json',
    language: 'json',
    code: `{
  "$schema": "https://appify.aerovex.net/schema.v1.json",
  "app": {
    "name": "Linear",
    "identifier": "com.linear.appify",
    "url": "https://linear.app",
    "version": "1.0.0",
    "window": {
      "width": 1280,
      "height": 820,
      "minWidth": 800,
      "minHeight": 600,
      "decorations": true,
      "transparent": false
    },
    "tray": {
      "enabled": true,
      "minimizeToTray": true,
      "unreadBadge": true
    },
    "security": {
      "sandbox": true,
      "allowDownloads": true,
      "sessionPartition": "linear-vault"
    }
  }
}`,
  },
  tray: {
    filename: 'tray_manager.rs',
    language: 'rust',
    code: `use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("show", "Open Window").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Minimize to Tray").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Appify").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &hide, &quit]).build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_tray_icon_event(|tray, event| {
            // Low-latency tray click handling (<14 ms)
        })
        .build(app)?;
    Ok(())
}`,
  },
  memfd: {
    filename: 'memfd_exec.rs',
    language: 'rust',
    code: `use nix::sys::memfd::{memfd_create, MemFdCreateFlag};
use nix::fcntl::{fcntl, FcntlArg, SealFlag};
use std::os::unix::io::AsRawFd;

pub fn execute_in_ram(payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let memfd = memfd_create(c"appify_isolated", MemFdCreateFlag::MFD_CLOEXEC | MemFdCreateFlag::MFD_ALLOW_SEALING)?;
    nix::unistd::write(memfd.as_raw_fd(), payload)?;
    // Seal against memory modification
    fcntl(memfd.as_raw_fd(), FcntlArg::F_ADD_SEALS(SealFlag::F_SEAL_WRITE | SealFlag::F_SEAL_SEAL))?;
    // Execute directly in RAM without leaving disk traces
    Ok(())
}`,
  },
}

function copyPipelineCode() {
  const item = pipelineSnippets[activePipelineTab.value]
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    navigator.clipboard.writeText(item.code)
    pipelineCopied.value = true
    setTimeout(() => {
      pipelineCopied.value = false
    }, 2000)
  }
}

// Copy fast CLI install command
const copiedInstall = ref(false)
function copyInstallCommand() {
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    navigator.clipboard.writeText('curl -fsSL https://appify.aerovex.net/install.sh | bash')
    copiedInstall.value = true
    setTimeout(() => {
      copiedInstall.value = false
    }, 2000)
  }
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('keydown', handleGlobalKeydown)
    document.addEventListener('click', handleClickOutside)
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('keydown', handleGlobalKeydown)
    document.removeEventListener('click', handleClickOutside)
  }
})

function getStatusToneClass(tone: string) {
  switch (tone) {
    case 'emerald': return 'appify-status-emerald'
    case 'cyan': return 'appify-status-cyan'
    case 'amber': return 'appify-status-amber'
    default: return 'appify-status-primary'
  }
}
</script>

<template>
  <div class="appify-landing-root">
    <!-- Atmospheric Ambient Glow -->
    <div class="nx-ambient-glow" aria-hidden="true"></div>

    <!-- Fixed Blur Navbar (Canonical Aerovex Architecture) -->
    <header class="nx-navbar">
      <div class="nx-navbar-inner">
        <!-- Brand Mark -->
        <div class="nx-navbar-brand-group">
          <a href="/" class="nx-navbar-brand">
            <svg class="nx-brand-icon" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
              <line x1="8" y1="21" x2="16" y2="21" />
              <line x1="12" y1="17" x2="12" y2="21" />
            </svg>
            <span class="nx-brand-title">APPIFY.</span>
            <span class="nx-brand-badge">BUNDLER</span>
            <span class="nx-telemetry-dot" title="Tauri 2.0 Engine Active"></span>
          </a>
        </div>

        <!-- Desktop Navigation Links -->
        <nav class="nx-navbar-links" aria-label="Main Navigation">
          <a href="#subsystems" class="nx-nav-link">Subsystems</a>
          <a href="#pipeline" class="nx-nav-link">Packaging Pipeline</a>
          <a href="#benchmarks" class="nx-nav-link">Benchmarks</a>
          <a href="/guide/presets" class="nx-nav-link">Presets</a>
          <a href="/guide/marketplace" class="nx-nav-link">Marketplace</a>

          <!-- Ecosystem Subdomains Dropdown -->
          <div class="nx-ecosystem-dropdown-wrap" ref="dropdownRef">
            <button
              type="button"
              class="nx-ecosystem-trigger"
              :class="{ 'nx-ecosystem-trigger-open': ecosystemOpen }"
              @click.stop="toggleEcosystem"
              aria-haspopup="true"
              :aria-expanded="ecosystemOpen"
            >
              <span>Ecosystem</span>
              <svg
                class="nx-ecosystem-chevron"
                :class="{ 'nx-ecosystem-chevron-open': ecosystemOpen }"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>

            <!-- Dropdown Panel -->
            <transition name="dropdown-fade">
              <div v-if="ecosystemOpen" class="nx-ecosystem-panel" @click.stop>
                <div class="nx-ecosystem-header">
                  <span class="nx-ecosystem-title">Aerovex Architecture</span>
                  <span class="nx-ecosystem-subdomain-note">*.aerovex.net</span>
                </div>

                <div class="nx-ecosystem-items">
                  <a href="https://aerovex.net" target="_blank" rel="noopener noreferrer" class="nx-ecosystem-item">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Aerovex Portal
                        <span class="nx-ecosystem-item-tag">Operations</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">Unified operations portal, multi-world sim &amp; fleet management</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>portal</span>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                    </div>
                  </a>

                  <a href="https://axiom.aerovex.net" target="_blank" rel="noopener noreferrer" class="nx-ecosystem-item">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Axiom
                        <span class="nx-ecosystem-item-tag">Silicon HDL</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">In-RAM Verilog/SystemVerilog simulator &amp; Cranelift JIT compiler</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>axiom</span>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                    </div>
                  </a>

                  <a href="https://phonon.aerovex.net" target="_blank" rel="noopener noreferrer" class="nx-ecosystem-item">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Phonon
                        <span class="nx-ecosystem-item-tag">Semiconductor TCAD</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">Universal electro-thermal SPICE &amp; 4.2K Cryo-CMOS simulator</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>phonon</span>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                    </div>
                  </a>

                  <a href="https://atoma.aerovex.net" target="_blank" rel="noopener noreferrer" class="nx-ecosystem-item">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Atoma
                        <span class="nx-ecosystem-item-tag">Quantum &amp; CAD</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">3D atomistic studio, 14 Bravais crystal lattices &amp; Dirac cones</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>atoma</span>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                    </div>
                  </a>

                  <a href="https://sonon.aerovex.net" target="_blank" rel="noopener noreferrer" class="nx-ecosystem-item">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Sonon
                        <span class="nx-ecosystem-item-tag">Acoustic DSP</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">Rotor noise notch filtering (&gt;86 dB) &amp; articulatory speech</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>sonon</span>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                    </div>
                  </a>

                  <a href="https://vexel.aerovex.net" target="_blank" rel="noopener noreferrer" class="nx-ecosystem-item">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Vexel Language
                        <span class="nx-ecosystem-item-tag">Compiler</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">Statically-typed systems language with LLVM 18 &amp; borrow checker</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>vexel</span>
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                    </div>
                  </a>

                  <div class="nx-ecosystem-item nx-ecosystem-item-active">
                    <div class="nx-ecosystem-item-left">
                      <div class="nx-ecosystem-item-name">
                        Appify
                        <span class="nx-ecosystem-item-badge">Active</span>
                      </div>
                      <div class="nx-ecosystem-item-desc">Modern web-to-native desktop application packager (Tauri 2.0)</div>
                    </div>
                    <div class="nx-ecosystem-subdomain-pill">
                      <span>appify</span>
                    </div>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </nav>

        <!-- Navbar Right Actions -->
        <div class="nx-navbar-actions">
          <a href="/guide/introduction" class="nx-nav-btn-primary">
            <span>Documentation</span>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="5" y1="12" x2="19" y2="12" />
              <polyline points="12 5 19 12 12 19" />
            </svg>
          </a>
          <a
            href="https://github.com/aerovexhq/appify"
            target="_blank"
            rel="noopener noreferrer"
            class="nx-nav-github"
            aria-label="GitHub Repository"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
            </svg>
          </a>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="appify-main">
      <!-- 00 // HERO SECTION -->
      <section id="home" class="nx-hero container">
        <div class="nx-hero-content">
          <!-- Status Pill -->
          <div class="nx-hero-eyebrow">
            <span class="nx-telemetry-dot"></span>
            <span>WEB-TO-NATIVE DESKTOP BUNDLER // TAURI 2.0 &amp; RUST</span>
          </div>

          <!-- Main Monolithic Headline -->
          <h1 class="nx-hero-title">
            <span>Transform Any Modern Web Application into a </span>
            <span class="nx-hero-title-accent">Sub-5MB Native Desktop Binary.</span>
          </h1>

          <!-- Elevator Pitch -->
          <p class="nx-hero-sub">
            Say goodbye to 150MB+ Electron bundles and runaway memory usage. Appify leverages Tauri 2.0, native OS WebKit/WebView2 engines, and anonymous RAM execution to package web applications into ultra-fast, isolated, and secure desktop executables.
          </p>

          <!-- Primary CTAs -->
          <div class="nx-hero-actions">
            <a href="/guide/introduction" class="nx-hero-cta-primary">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
              </svg>
              <span>Explore Documentation</span>
            </a>

            <a href="/guide/presets" class="nx-hero-cta-secondary">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="7" height="7" />
                <rect x="14" y="3" width="7" height="7" />
                <rect x="14" y="14" width="7" height="7" />
                <rect x="3" y="14" width="7" height="7" />
              </svg>
              <span>Curated Presets</span>
            </a>

            <a href="/guide/cli" class="nx-hero-cta-secondary">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="4 17 10 11 4 5" />
                <line x1="12" y1="19" x2="20" y2="19" />
              </svg>
              <span>CLI Cheat Sheet</span>
            </a>
          </div>

          <!-- Live Metrics Ticker -->
          <div class="nx-hero-metrics">
            <div class="nx-hero-metric-item">
              <span class="nx-metric-val">&lt;4.8 MB</span>
              <span class="nx-metric-lbl">Production Binary Size</span>
            </div>
            <div class="nx-hero-metric-item">
              <span class="nx-metric-val">22.4 MB</span>
              <span class="nx-metric-lbl">Idle RAM Footprint</span>
            </div>
            <div class="nx-hero-metric-item">
              <span class="nx-metric-val">124 ms</span>
              <span class="nx-metric-lbl">Cold Startup Latency</span>
            </div>
            <div class="nx-hero-metric-item">
              <span class="nx-metric-val">0 Byte</span>
              <span class="nx-metric-lbl">memfd_create Disk Writes</span>
            </div>
          </div>
        </div>

        <!-- Hero Terminal Viewport -->
        <div class="nx-hero-viewport">
          <div class="nx-hero-viewport-topbar">
            <div class="nx-frame-dots">
              <span class="nx-dot nx-dot-red"></span>
              <span class="nx-dot nx-dot-yellow"></span>
              <span class="nx-dot nx-dot-green"></span>
            </div>
            <div class="nx-frame-title">
              APPIFY CLI BUILD PIPELINE // TAURI 2.0 (x86_64-linux-gnu)
            </div>
            <div class="nx-frame-telemetry">
              <span class="nx-telemetry-dot"></span>
              <span>LTO: STRIPPED (4.8 MB)</span>
            </div>
          </div>

          <div class="nx-hero-viewport-body">
            <div class="nx-terminal-content">
              <div class="nx-term-line nx-term-dim">$ appify pack https://linear.app --name "Linear" --tray --isolated</div>
              <div class="nx-term-line nx-term-info">[INFO] Resolving WebKitGTK / WRY host runtime bindings...</div>
              <div class="nx-term-line">[1/4] Generating isolated cookie partition: sha256(linear.app)... done (4ms)</div>
              <div class="nx-term-line">[2/4] Injecting system tray daemon and autostart desktop entry... done (12ms)</div>
              <div class="nx-term-line">[3/4] Compressing assets with zstd streaming (ratio: 3.82:1)... done (180ms)</div>
              <div class="nx-term-line nx-term-emerald">[4/4] Finalizing stripped standalone binary: ./dist/linear (4.8 MB)</div>
              <div class="nx-term-line nx-term-accent">SUCCESS: Native desktop app ready. Cold launch latency: 124 ms. Zero Electron bloat.</div>
            </div>
          </div>
        </div>
      </section>

      <!-- 01 // BALANCED BENTO GRID WITH SPOTLIGHT -->
      <section id="subsystems" class="nx-ecosystem-section container">
        <div class="nx-ecosystem-header-wrap">
          <div class="nx-ecosystem-eyebrow">
            01 // CORE ARCHITECTURAL SUBSYSTEMS
          </div>
          <h2 class="nx-ecosystem-title">
            Engineered for Extreme Efficiency
          </h2>
          <p class="nx-ecosystem-lead">
            Modular, high-performance web-to-native architecture built with pure Rust. Zero dynamic overhead, deterministic resource limits, and seamless integration with Linux, macOS, and Windows.
          </p>

          <!-- Domain Filter Tabs -->
          <div class="nx-ecosystem-filter-bar" role="tablist">
            <button
              type="button"
              role="tab"
              :aria-selected="activeFilter === 'all'"
              class="nx-filter-tab"
              :class="{ 'nx-filter-tab-active': activeFilter === 'all' }"
              @click="activeFilter = 'all'"
            >
              <span>All Subsystems</span>
              <span class="nx-filter-count">5</span>
            </button>
            <button
              type="button"
              role="tab"
              :aria-selected="activeFilter === 'runtime'"
              class="nx-filter-tab"
              :class="{ 'nx-filter-tab-active': activeFilter === 'runtime' }"
              @click="activeFilter = 'runtime'"
            >
              <span>Host Webview &amp; Security</span>
              <span class="nx-filter-count">2</span>
            </button>
            <button
              type="button"
              role="tab"
              :aria-selected="activeFilter === 'bundling'"
              class="nx-filter-tab"
              :class="{ 'nx-filter-tab-active': activeFilter === 'bundling' }"
              @click="activeFilter = 'bundling'"
            >
              <span>Packaging &amp; Presets</span>
              <span class="nx-filter-count">2</span>
            </button>
            <button
              type="button"
              role="tab"
              :aria-selected="activeFilter === 'ipc'"
              class="nx-filter-tab"
              :class="{ 'nx-filter-tab-active': activeFilter === 'ipc' }"
              @click="activeFilter = 'ipc'"
            >
              <span>Tray &amp; IPC Bridge</span>
              <span class="nx-filter-count">1</span>
            </button>
          </div>
        </div>

        <!-- 2-Column Balanced Bento Grid with Radial Mouse Spotlight -->
        <div class="nx-bento-2col">
          <article
            v-for="(subsystem, idx) in filteredSubsystems"
            :key="subsystem.id"
            :ref="(el) => setCardRef(el, idx)"
            class="appify-spotlight-card nx-project-card"
            @mousemove="handleMouseMove($event, idx)"
            @mouseleave="handleMouseLeave(idx)"
          >
            <!-- Card Header -->
            <div class="nx-project-card-header">
              <div class="nx-project-meta">
                <span class="nx-project-tier-tag">{{ subsystem.tierLabel }}</span>
                <span class="nx-project-category">{{ subsystem.category }}</span>
              </div>
              <div class="nx-project-status-badge" :class="getStatusToneClass(subsystem.statusTone)">
                <span class="nx-status-dot"></span>
                <span>{{ subsystem.status }}</span>
              </div>
            </div>

            <!-- Card Body -->
            <div class="nx-project-body">
              <div class="nx-project-title-row">
                <h3 class="nx-project-name">{{ subsystem.name }}</h3>
              </div>
              <p class="nx-project-tagline">{{ subsystem.tagline }}</p>

              <!-- Empirical Benchmark Metric Well -->
              <div class="nx-project-metric-box">
                <div class="nx-project-metric-val">{{ subsystem.metric }}</div>
                <div class="nx-project-metric-desc">{{ subsystem.metricDetail }}</div>
              </div>

              <p class="nx-project-desc">{{ subsystem.description }}</p>

              <!-- Architecture Highlights -->
              <ul class="nx-project-highlights">
                <li v-for="(hl, hIdx) in subsystem.highlights" :key="hIdx" class="nx-project-highlight-item">
                  <span class="nx-highlight-bullet"></span>
                  <span>{{ hl }}</span>
                </li>
              </ul>
            </div>

            <!-- Card Footer -->
            <div class="nx-project-card-footer">
              <button
                type="button"
                class="nx-project-btn-deepdive"
                @click="openArchitecture(subsystem)"
              >
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="12 2 2 7 12 12 22 7 12 2" />
                  <polyline points="2 17 12 22 22 17" />
                  <polyline points="2 12 12 17 22 12" />
                </svg>
                <span>Inspect Architecture</span>
              </button>

              <a :href="subsystem.docsLink" class="nx-project-btn-subdomain">
                <span>View Documentation</span>
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <line x1="5" y1="12" x2="19" y2="12" />
                  <polyline points="12 5 19 12 12 19" />
                </svg>
              </a>
            </div>
          </article>
        </div>
      </section>

      <!-- 02 // PACKAGING PIPELINE SECTION -->
      <section id="pipeline" class="nx-section container">
        <div class="nx-section-header">
          <div class="nx-section-eyebrow">02 // DATAFLOW &amp; PACKAGING PIPELINE</div>
          <h2 class="nx-section-title">From Web Application URL to Standalone Binary</h2>
          <p class="nx-section-lead">
            Inspect the high-speed compilation and isolation flow converting web code into sub-5MB native executables without bundled Chromium dependencies.
          </p>
        </div>

        <div class="nx-pipeline-showcase">
          <!-- Pipeline Tabs -->
          <div class="nx-pipeline-tabs">
            <button
              type="button"
              class="nx-pipe-tab"
              :class="{ 'nx-pipe-tab-active': activePipelineTab === 'cli' }"
              @click="activePipelineTab = 'cli'"
            >
              <span>1-Command CLI</span>
            </button>
            <button
              type="button"
              class="nx-pipe-tab"
              :class="{ 'nx-pipe-tab-active': activePipelineTab === 'manifest' }"
              @click="activePipelineTab = 'manifest'"
            >
              <span>Manifest Schema</span>
            </button>
            <button
              type="button"
              class="nx-pipe-tab"
              :class="{ 'nx-pipe-tab-active': activePipelineTab === 'tray' }"
              @click="activePipelineTab = 'tray'"
            >
              <span>Tray Daemon (Rust)</span>
            </button>
            <button
              type="button"
              class="nx-pipe-tab"
              :class="{ 'nx-pipe-tab-active': activePipelineTab === 'memfd' }"
              @click="activePipelineTab = 'memfd'"
            >
              <span>RAM Execution (memfd)</span>
            </button>
          </div>

          <!-- Code Display Box -->
          <div class="nx-code-box">
            <div class="nx-code-box-header">
              <span class="nx-code-filename">{{ pipelineSnippets[activePipelineTab].filename }}</span>
              <button type="button" class="nx-copy-btn" @click="copyPipelineCode">
                <span>{{ pipelineCopied ? 'Copied' : 'Copy Snippet' }}</span>
              </button>
            </div>
            <pre class="nx-code-body"><code>{{ pipelineSnippets[activePipelineTab].code }}</code></pre>
          </div>
        </div>
      </section>

      <!-- 03 // BENCHMARK COMPARISON MATRIX -->
      <section id="benchmarks" class="nx-section container">
        <div class="nx-section-header">
          <div class="nx-section-eyebrow">03 // EMPIRICAL BENCHMARKS</div>
          <h2 class="nx-section-title">Appify vs Traditional Desktop Bundlers</h2>
          <p class="nx-section-lead">
            Rigorous side-by-side performance comparison on identical Linux x86_64 hardware.
          </p>
        </div>

        <div class="nx-benchmark-table-wrap">
          <table class="nx-benchmark-table">
            <thead>
              <tr>
                <th class="nx-th-metric">Metric / Dimension</th>
                <th class="nx-th-appify">Appify (Tauri 2.0)</th>
                <th>Electron 32</th>
                <th>NW.js 0.88</th>
                <th>Nativefier</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, rIdx) in BENCHMARK_COMPARISONS" :key="rIdx">
                <td class="nx-td-metric">{{ row.metric }}</td>
                <td class="nx-td-appify">
                  <span class="nx-appify-badge">{{ row.appify }}</span>
                </td>
                <td>{{ row.electron }}</td>
                <td>{{ row.nwjs }}</td>
                <td>{{ row.nativefier }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <!-- 04 // FAST ONE-LINE INSTALLER -->
      <section class="nx-section container">
        <div class="nx-installer-card">
          <div class="nx-installer-left">
            <div class="nx-installer-eyebrow">FAST INSTALLATION // LINUX &amp; MACOS</div>
            <h3 class="nx-installer-title">Install Appify CLI in Seconds</h3>
            <p class="nx-installer-desc">
              Single-line curl installer compiles zero runtime dependencies and installs appify directly into your local path.
            </p>
          </div>
          <div class="nx-installer-right">
            <div class="nx-installer-cmd-box">
              <code class="nx-installer-cmd">curl -fsSL https://appify.aerovex.net/install.sh | bash</code>
              <button type="button" class="nx-copy-btn" @click="copyInstallCommand">
                <span>{{ copiedInstall ? 'Copied' : 'Copy' }}</span>
              </button>
            </div>
          </div>
        </div>
      </section>
    </main>

    <!-- Interactive Architecture Modal -->
    <transition name="modal-fade">
      <div v-if="selectedSubsystem" class="nx-modal-backdrop" @click.self="closeArchitecture">
        <div class="nx-modal-window" role="dialog" aria-modal="true" :aria-label="selectedSubsystem.name">
          <!-- Modal Header -->
          <div class="nx-modal-header">
            <div class="nx-modal-header-meta">
              <span class="nx-project-tier-tag">{{ selectedSubsystem.tierLabel }}</span>
              <span class="nx-project-category">{{ selectedSubsystem.category }}</span>
            </div>
            <button type="button" class="nx-modal-close-btn" @click="closeArchitecture" aria-label="Close Modal">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          <!-- Modal Body -->
          <div class="nx-modal-body">
            <h3 class="nx-modal-title">{{ selectedSubsystem.name }}</h3>
            <p class="nx-modal-tagline">{{ selectedSubsystem.tagline }}</p>

            <!-- Execution Pipeline ASCII -->
            <div class="nx-modal-section">
              <div class="nx-modal-section-title">01 // EXECUTION PIPELINE</div>
              <pre class="nx-modal-ascii"><code>{{ selectedSubsystem.architectureDetails.pipelineAscii }}</code></pre>
            </div>

            <!-- Empirical Benchmarks -->
            <div class="nx-modal-section">
              <div class="nx-modal-section-title">02 // EMPIRICAL BENCHMARKS</div>
              <div class="nx-modal-bench-grid">
                <div
                  v-for="(bench, bIdx) in selectedSubsystem.architectureDetails.benchmarks"
                  :key="bIdx"
                  class="nx-modal-bench-item"
                >
                  <div class="nx-modal-bench-val">{{ bench.value }} <span class="nx-modal-bench-unit">{{ bench.unit }}</span></div>
                  <div class="nx-modal-bench-lbl">{{ bench.label }}</div>
                  <div class="nx-modal-bench-base">Baseline: {{ bench.baseline }}</div>
                </div>
              </div>
            </div>

            <!-- Mathematical Formulation -->
            <div class="nx-modal-section">
              <div class="nx-modal-section-title">03 // SYSTEM INVARIANT &amp; FORMULATION</div>
              <div class="nx-modal-math-box">
                <code>{{ selectedSubsystem.architectureDetails.mathematicalFormulation }}</code>
              </div>
            </div>

            <!-- Implementation Snippet -->
            <div class="nx-modal-section">
              <div class="nx-modal-section-title-row">
                <span class="nx-modal-section-title">04 // CORE IMPLEMENTATION INTERFACE</span>
                <button type="button" class="nx-copy-btn" @click="copySnippetText(selectedSubsystem.architectureDetails.codeSnippet.code)">
                  <span>{{ copiedSnippet ? 'Copied' : 'Copy Code' }}</span>
                </button>
              </div>
              <div class="nx-code-box">
                <div class="nx-code-box-header">
                  <span class="nx-code-filename">{{ selectedSubsystem.architectureDetails.codeSnippet.filename }}</span>
                  <span class="nx-code-lang">{{ selectedSubsystem.architectureDetails.codeSnippet.language }}</span>
                </div>
                <pre class="nx-code-body"><code>{{ selectedSubsystem.architectureDetails.codeSnippet.code }}</code></pre>
              </div>
            </div>

            <!-- Architectural Tradeoffs -->
            <div class="nx-modal-section">
              <div class="nx-modal-section-title">05 // ARCHITECTURAL TRADEOFFS</div>
              <p class="nx-modal-tradeoffs">{{ selectedSubsystem.architectureDetails.tradeoffs }}</p>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <!-- Unified Footer Directory -->
    <footer class="nx-footer">
      <div class="nx-footer-inner container">
        <div class="nx-footer-top">
          <div class="nx-footer-brand-col">
            <div class="nx-footer-brand">
              <span class="nx-footer-logo">APPIFY.</span>
              <span class="nx-footer-badge">TAURI 2.0</span>
            </div>
            <p class="nx-footer-tagline">
              Modern web-to-native desktop application bundler and packaging framework by Aerovex HQ.
            </p>
          </div>

          <div class="nx-footer-nav-col">
            <span class="nx-footer-col-title">Framework</span>
            <a href="/guide/introduction" class="nx-footer-link">Getting Started</a>
            <a href="/guide/installation" class="nx-footer-link">Installation</a>
            <a href="/guide/presets" class="nx-footer-link">Presets Catalog</a>
            <a href="/guide/cli" class="nx-footer-link">CLI Commands</a>
          </div>

          <div class="nx-footer-nav-col">
            <span class="nx-footer-col-title">Ecosystem</span>
            <a href="https://aerovex.net" class="nx-footer-link">Aerovex Portal</a>
            <a href="https://axiom.aerovex.net" class="nx-footer-link">Axiom Silicon HDL</a>
            <a href="https://phonon.aerovex.net" class="nx-footer-link">Phonon EDA</a>
            <a href="https://atoma.aerovex.net" class="nx-footer-link">Atoma CAD</a>
            <a href="https://sonon.aerovex.net" class="nx-footer-link">Sonon DSP</a>
            <a href="https://vexel.aerovex.net" class="nx-footer-link">Vexel Language</a>
          </div>

          <div class="nx-footer-nav-col">
            <span class="nx-footer-col-title">Community</span>
            <a href="https://github.com/aerovexhq/appify" class="nx-footer-link">GitHub</a>
            <a href="https://appify.aerovex.net" class="nx-footer-link">Documentation</a>
            <a href="https://aerovex.net" class="nx-footer-link">Aerovex HQ</a>
          </div>
        </div>

        <div class="nx-footer-bottom">
          <div class="nx-footer-copy">
            Copyright (c) 2026 Aerovex Engineers &amp; Appify Contributors. Released under MIT License.
          </div>
          <div class="nx-footer-status">
            <span class="nx-telemetry-dot"></span>
            <span>TAURI 2.0 // SUB-5MB CORE // HARDWARE ACCELERATED</span>
          </div>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
/* Scoped styles matching the Aerovex Linear portal design system */
.appify-landing-root {
  background-color: #08090a;
  color: #f4f4f5;
  min-height: 100vh;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  letter-spacing: -0.015em;
  position: relative;
  overflow-x: hidden;
}

.container {
  max-width: 1240px;
  margin: 0 auto;
  padding: 0 24px;
}

/* Ambient glow */
.nx-ambient-glow {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 1000px;
  height: 480px;
  background: radial-gradient(circle, rgba(94, 106, 210, 0.12) 0%, rgba(56, 189, 248, 0.04) 40%, transparent 70%);
  pointer-events: none;
  z-index: 0;
}

/* Navbar */
.nx-navbar {
  position: sticky;
  top: 0;
  z-index: 50;
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  background-color: rgba(8, 9, 10, 0.85);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.nx-navbar-inner {
  max-width: 1240px;
  margin: 0 auto;
  padding: 0 24px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.nx-navbar-brand-group {
  display: flex;
  align-items: center;
}

.nx-navbar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  text-decoration: none;
  color: #fff;
}

.nx-brand-icon {
  color: #5e6ad2;
}

.nx-brand-title {
  font-size: 17px;
  font-weight: 800;
  letter-spacing: -0.035em;
  color: #ffffff;
}

.nx-brand-badge {
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(94, 106, 210, 0.15);
  color: #818cf8;
  border: 1px solid rgba(94, 106, 210, 0.3);
}

.nx-telemetry-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.6);
  display: inline-block;
  animation: pulse-dot 2.5s infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.45; transform: scale(0.85); }
}

.nx-navbar-links {
  display: flex;
  align-items: center;
  gap: 24px;
}

.nx-nav-link {
  font-size: 13.5px;
  font-weight: 500;
  color: #a1a1aa;
  text-decoration: none;
  transition: color 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

.nx-nav-link:hover {
  color: #ffffff;
}

.nx-ecosystem-dropdown-wrap {
  position: relative;
}

.nx-ecosystem-trigger {
  display: flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  font-size: 13.5px;
  font-weight: 500;
  color: #a1a1aa;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 6px;
  transition: all 0.18s;
}

.nx-ecosystem-trigger:hover, .nx-ecosystem-trigger-open {
  color: #ffffff;
  background: rgba(255, 255, 255, 0.05);
}

.nx-ecosystem-chevron {
  transition: transform 0.2s;
}

.nx-ecosystem-chevron-open {
  transform: rotate(180deg);
}

.nx-ecosystem-panel {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 380px;
  background: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
  z-index: 100;
}

.nx-ecosystem-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  margin-bottom: 8px;
}

.nx-ecosystem-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #71717a;
}

.nx-ecosystem-subdomain-note {
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  color: #5e6ad2;
}

.nx-ecosystem-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nx-ecosystem-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 8px;
  text-decoration: none;
  transition: background 0.15s;
}

.nx-ecosystem-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.nx-ecosystem-item-active {
  background: rgba(94, 106, 210, 0.1);
  border: 1px solid rgba(94, 106, 210, 0.25);
}

.nx-ecosystem-item-name {
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
  display: flex;
  align-items: center;
  gap: 8px;
}

.nx-ecosystem-item-tag {
  font-size: 9.5px;
  font-weight: 500;
  color: #71717a;
}

.nx-ecosystem-item-badge {
  font-size: 9px;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.nx-ecosystem-item-desc {
  font-size: 11px;
  color: #71717a;
  margin-top: 2px;
}

.nx-ecosystem-subdomain-pill {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  padding: 3px 7px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.04);
  color: #a1a1aa;
  display: flex;
  align-items: center;
  gap: 4px;
}

.nx-navbar-actions {
  display: flex;
  align-items: center;
  gap: 14px;
}

.nx-nav-btn-primary {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #5e6ad2;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  padding: 7px 14px;
  border-radius: 6px;
  text-decoration: none;
  transition: all 0.18s;
}

.nx-nav-btn-primary:hover {
  background: #4f5bc4;
  box-shadow: 0 0 15px rgba(94, 106, 210, 0.4);
}

.nx-nav-github {
  color: #a1a1aa;
  padding: 6px;
  border-radius: 6px;
  transition: color 0.18s;
  display: flex;
  align-items: center;
}

.nx-nav-github:hover {
  color: #ffffff;
}

/* Hero Section */
.nx-hero {
  padding-top: 72px;
  padding-bottom: 72px;
  display: flex;
  flex-direction: column;
  gap: 40px;
}

.nx-hero-eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  color: #a1a1aa;
  padding: 4px 10px;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  margin-bottom: 20px;
}

.nx-hero-title {
  font-size: 46px;
  line-height: 1.12;
  font-weight: 800;
  letter-spacing: -0.035em;
  color: #ffffff;
  max-width: 900px;
  margin-bottom: 20px;
}

.nx-hero-title-accent {
  background: linear-gradient(135deg, #818cf8 0%, #38bdf8 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nx-hero-sub {
  font-size: 16.5px;
  line-height: 1.6;
  color: #a1a1aa;
  max-width: 780px;
  margin-bottom: 28px;
}

.nx-hero-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 36px;
}

.nx-hero-cta-primary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: #5e6ad2;
  color: #ffffff;
  font-size: 14px;
  font-weight: 600;
  padding: 10px 20px;
  border-radius: 8px;
  text-decoration: none;
  transition: all 0.18s;
}

.nx-hero-cta-primary:hover {
  background: #4f5bc4;
  box-shadow: 0 0 20px rgba(94, 106, 210, 0.45);
}

.nx-hero-cta-secondary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.04);
  color: #d4d4d8;
  border: 1px solid rgba(255, 255, 255, 0.08);
  font-size: 14px;
  font-weight: 500;
  padding: 10px 18px;
  border-radius: 8px;
  text-decoration: none;
  transition: all 0.18s;
}

.nx-hero-cta-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.nx-hero-metrics {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  max-width: 900px;
  padding-top: 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.nx-hero-metric-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nx-metric-val {
  font-family: 'JetBrains Mono', monospace;
  font-size: 22px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: -0.02em;
}

.nx-metric-lbl {
  font-size: 11.5px;
  color: #71717a;
}

/* Terminal Viewport */
.nx-hero-viewport {
  background: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.5);
}

.nx-hero-viewport-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: #15171a;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.nx-frame-dots {
  display: flex;
  align-items: center;
  gap: 6px;
}

.nx-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.nx-dot-red { background: #ef4444; }
.nx-dot-yellow { background: #f59e0b; }
.nx-dot-green { background: #10b981; }

.nx-frame-title {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #71717a;
  letter-spacing: 0.04em;
}

.nx-frame-telemetry {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #10b981;
}

.nx-hero-viewport-body {
  padding: 20px;
  background: #08090a;
}

.nx-terminal-content {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12.5px;
  line-height: 1.8;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.nx-term-line {
  color: #a1a1aa;
}

.nx-term-dim { color: #71717a; }
.nx-term-info { color: #38bdf8; }
.nx-term-emerald { color: #10b981; font-weight: 600; }
.nx-term-accent { color: #818cf8; font-weight: 600; }

/* Bento Grid */
.nx-ecosystem-section {
  padding: 72px 24px;
}

.nx-ecosystem-header-wrap {
  margin-bottom: 40px;
}

.nx-ecosystem-eyebrow {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #5e6ad2;
  font-weight: 600;
  letter-spacing: 0.08em;
  margin-bottom: 8px;
}

.nx-ecosystem-title {
  font-size: 32px;
  font-weight: 800;
  letter-spacing: -0.035em;
  color: #ffffff;
  margin-bottom: 12px;
}

.nx-ecosystem-lead {
  font-size: 15px;
  line-height: 1.6;
  color: #a1a1aa;
  max-width: 700px;
  margin-bottom: 24px;
}

.nx-ecosystem-filter-bar {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.nx-filter-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: #a1a1aa;
  font-size: 12.5px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.nx-filter-tab:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #ffffff;
}

.nx-filter-tab-active {
  background: rgba(94, 106, 210, 0.15);
  border-color: rgba(94, 106, 210, 0.4);
  color: #ffffff;
}

.nx-filter-count {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.08);
}

.nx-bento-2col {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

/* Spotlight Card */
.appify-spotlight-card {
  position: relative;
  background-color: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  overflow: hidden;
  transition: border-color 0.2s, transform 0.2s;
}

.appify-spotlight-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: radial-gradient(
    400px circle at var(--mouse-x, -999px) var(--mouse-y, -999px),
    rgba(94, 106, 210, 0.08),
    transparent 80%
  );
  opacity: var(--spotlight-opacity, 0);
  pointer-events: none;
  transition: opacity 0.25s;
}

.appify-spotlight-card:hover {
  border-color: rgba(255, 255, 255, 0.16);
}

.nx-project-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.nx-project-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.nx-project-tier-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10.5px;
  color: #71717a;
  font-weight: 600;
}

.nx-project-category {
  font-size: 11px;
  color: #a1a1aa;
}

.nx-project-status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 4px;
}

.appify-status-primary {
  background: rgba(94, 106, 210, 0.12);
  color: #818cf8;
  border: 1px solid rgba(94, 106, 210, 0.3);
}

.appify-status-emerald {
  background: rgba(16, 185, 129, 0.12);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.appify-status-cyan {
  background: rgba(56, 189, 248, 0.12);
  color: #38bdf8;
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.appify-status-amber {
  background: rgba(245, 158, 11, 0.12);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.nx-status-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: currentColor;
}

.nx-project-name {
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.025em;
  color: #ffffff;
  margin-bottom: 6px;
}

.nx-project-tagline {
  font-size: 13px;
  color: #a1a1aa;
  margin-bottom: 14px;
}

.nx-project-metric-box {
  background: #15171a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 10px 14px;
  margin-bottom: 14px;
}

.nx-project-metric-val {
  font-family: 'JetBrains Mono', monospace;
  font-size: 15px;
  font-weight: 700;
  color: #ffffff;
}

.nx-project-metric-desc {
  font-size: 11px;
  color: #71717a;
  margin-top: 2px;
}

.nx-project-desc {
  font-size: 13px;
  line-height: 1.55;
  color: #d4d4d8;
  margin-bottom: 16px;
}

.nx-project-highlights {
  list-style: none;
  padding: 0;
  margin: 0 0 20px 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.nx-project-highlight-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 12px;
  color: #a1a1aa;
}

.nx-highlight-bullet {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #5e6ad2;
  margin-top: 6px;
  flex-shrink: 0;
}

.nx-project-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.nx-project-btn-deepdive {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: #ffffff;
  font-size: 12px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.nx-project-btn-deepdive:hover {
  background: rgba(94, 106, 210, 0.2);
  border-color: rgba(94, 106, 210, 0.4);
}

.nx-project-btn-subdomain {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #71717a;
  font-size: 12px;
  text-decoration: none;
  transition: color 0.15s;
}

.nx-project-btn-subdomain:hover {
  color: #ffffff;
}

/* Pipeline Section */
.nx-section {
  padding: 64px 24px;
}

.nx-section-header {
  margin-bottom: 32px;
}

.nx-section-eyebrow {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #5e6ad2;
  font-weight: 600;
  letter-spacing: 0.08em;
  margin-bottom: 6px;
}

.nx-section-title {
  font-size: 28px;
  font-weight: 800;
  letter-spacing: -0.03em;
  color: #ffffff;
  margin-bottom: 10px;
}

.nx-section-lead {
  font-size: 15px;
  line-height: 1.55;
  color: #a1a1aa;
  max-width: 680px;
}

.nx-pipeline-showcase {
  background: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  overflow: hidden;
}

.nx-pipeline-tabs {
  display: flex;
  background: #15171a;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  overflow-x: auto;
}

.nx-pipe-tab {
  background: transparent;
  border: none;
  padding: 12px 18px;
  font-size: 13px;
  font-weight: 500;
  color: #71717a;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.nx-pipe-tab:hover {
  color: #ffffff;
}

.nx-pipe-tab-active {
  color: #ffffff;
  background: #0f1012;
  border-bottom: 2px solid #5e6ad2;
}

.nx-code-box {
  background: #08090a;
}

.nx-code-box-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.nx-code-filename {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: #a1a1aa;
}

.nx-code-lang {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #5e6ad2;
  text-transform: uppercase;
}

.nx-copy-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: #a1a1aa;
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.nx-copy-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.nx-code-body {
  margin: 0;
  padding: 16px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12.5px;
  line-height: 1.6;
  color: #e4e4e7;
  overflow-x: auto;
}

/* Benchmark Table */
.nx-benchmark-table-wrap {
  background: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  overflow-x: auto;
}

.nx-benchmark-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  font-size: 13px;
}

.nx-benchmark-table th {
  padding: 12px 16px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #71717a;
  background: #15171a;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.nx-th-appify {
  color: #818cf8 !important;
}

.nx-benchmark-table td {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  color: #a1a1aa;
}

.nx-td-metric {
  font-weight: 600;
  color: #ffffff;
}

.nx-td-appify {
  background: rgba(94, 106, 210, 0.05);
}

.nx-appify-badge {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  color: #818cf8;
}

/* Fast Installer Card */
.nx-installer-card {
  background: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 28px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 32px;
  flex-wrap: wrap;
}

.nx-installer-eyebrow {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  color: #5e6ad2;
  letter-spacing: 0.06em;
  margin-bottom: 6px;
}

.nx-installer-title {
  font-size: 22px;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 6px;
}

.nx-installer-desc {
  font-size: 13.5px;
  color: #a1a1aa;
  max-width: 500px;
}

.nx-installer-cmd-box {
  background: #08090a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 10px 14px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.nx-installer-cmd {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  color: #38bdf8;
}

/* Modal */
.nx-modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.nx-modal-window {
  background: #0f1012;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 14px;
  max-width: 840px;
  width: 100%;
  max-height: 88vh;
  overflow-y: auto;
  box-shadow: 0 32px 64px rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
}

.nx-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  position: sticky;
  top: 0;
  background: #0f1012;
  z-index: 10;
}

.nx-modal-header-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.nx-modal-close-btn {
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: color 0.15s;
}

.nx-modal-close-btn:hover {
  color: #ffffff;
}

.nx-modal-body {
  padding: 24px;
}

.nx-modal-title {
  font-size: 24px;
  font-weight: 800;
  color: #ffffff;
  margin-bottom: 6px;
}

.nx-modal-tagline {
  font-size: 14px;
  color: #a1a1aa;
  margin-bottom: 24px;
}

.nx-modal-section {
  margin-bottom: 24px;
}

.nx-modal-section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.nx-modal-section-title {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  color: #5e6ad2;
  letter-spacing: 0.06em;
  margin-bottom: 8px;
}

.nx-modal-ascii {
  background: #08090a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 14px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: #a1a1aa;
  overflow-x: auto;
}

.nx-modal-bench-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.nx-modal-bench-item {
  background: #15171a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 12px;
}

.nx-modal-bench-val {
  font-family: 'JetBrains Mono', monospace;
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
}

.nx-modal-bench-unit {
  font-size: 12px;
  color: #71717a;
}

.nx-modal-bench-lbl {
  font-size: 11px;
  color: #a1a1aa;
  margin-top: 4px;
}

.nx-modal-bench-base {
  font-size: 10px;
  color: #71717a;
  margin-top: 2px;
}

.nx-modal-math-box {
  background: #08090a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 12px 16px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  color: #38bdf8;
}

.nx-modal-tradeoffs {
  font-size: 13px;
  line-height: 1.6;
  color: #a1a1aa;
}

/* Footer */
.nx-footer {
  background: #08090a;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding: 64px 0 32px;
  margin-top: 48px;
}

.nx-footer-top {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr;
  gap: 40px;
  margin-bottom: 48px;
}

.nx-footer-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.nx-footer-logo {
  font-size: 16px;
  font-weight: 800;
  letter-spacing: -0.035em;
  color: #ffffff;
}

.nx-footer-badge {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9.5px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(94, 106, 210, 0.15);
  color: #818cf8;
}

.nx-footer-tagline {
  font-size: 12.5px;
  line-height: 1.6;
  color: #71717a;
  max-width: 320px;
}

.nx-footer-nav-col {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.nx-footer-col-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #ffffff;
  margin-bottom: 4px;
}

.nx-footer-link {
  font-size: 12.5px;
  color: #71717a;
  text-decoration: none;
  transition: color 0.15s;
}

.nx-footer-link:hover {
  color: #ffffff;
}

.nx-footer-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-wrap: wrap;
  gap: 16px;
}

.nx-footer-copy {
  font-size: 11.5px;
  color: #71717a;
}

.nx-footer-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #10b981;
}

/* Transitions */
.dropdown-fade-enter-active, .dropdown-fade-leave-active {
  transition: opacity 0.18s, transform 0.18s;
}
.dropdown-fade-enter-from, .dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.modal-fade-enter-active, .modal-fade-leave-active {
  transition: opacity 0.2s, transform 0.2s;
}
.modal-fade-enter-from, .modal-fade-leave-to {
  opacity: 0;
}

@media (max-width: 768px) {
  .nx-bento-2col {
    grid-template-columns: 1fr;
  }
  .nx-hero-metrics {
    grid-template-columns: repeat(2, 1fr);
  }
  .nx-footer-top {
    grid-template-columns: 1fr;
    gap: 24px;
  }
}
</style>
