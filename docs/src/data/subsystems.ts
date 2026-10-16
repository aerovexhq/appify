export interface Subsystem {
  id: string;
  name: string;
  domain: string;
  tierLabel: string;
  category: string;
  status: string;
  statusTone: 'emerald' | 'cyan' | 'amber' | 'indigo' | 'purple';
  tagline: string;
  metric: string;
  metricDetail: string;
  description: string;
  highlights: string[];
  tags: string[];
  docSectionId: string;
  architectureDetails: {
    pipelineDiagram: string;
    tradeoffs: { choice: string; why: string }[];
    codeSnippet: {
      language: string;
      title: string;
      code: string;
    };
  };
}

export const APPIFY_SUBSYSTEMS: Subsystem[] = [
  {
    id: 'single_binary_bundler',
    name: 'Single-Binary Native Desktop Bundler',
    domain: 'bundling',
    tierLabel: 'TIER 1 // TAURI 2.0 CORE',
    category: 'Application Packaging',
    status: 'ACTIVE',
    statusTone: 'purple',
    tagline: 'Packages web assets and native Rust core into ultra-compact sub-5MB standalone binaries.',
    metric: '<4.8 MB',
    metricDetail: 'Mean Standalone Native Executable Size on Linux (x86_64) and macOS (ARM64)',
    description: 'Compresses HTML, CSS, JavaScript, WebAssembly, and native Rust backend logic into a single self-contained native executable with zero Electron or Chromium bloat.',
    highlights: [
      'Eliminates bundled Chromium runtimes by utilizing OS-native WebViews (WebKitGTK, WKWebView, WebView2)',
      'Cross-compiles for Linux AppImage/deb, macOS DMG/app, and Windows MSI/exe',
      'In-RAM asset decompression and zero-disk temp file generation'
    ],
    tags: ['Tauri 2.0', 'Single-Binary', '<5MB Executable', 'WebKitGTK', 'WebView2'],
    docSectionId: 'single_binary_architecture',
    architectureDetails: {
      pipelineDiagram: `Web Distribution Assets (Vite / React / WASM)
           │
           ▼
[Appify Asset Inliner & Compiler] ── Static Resource Compression
           │
           ▼
[Native OS Shell Packaging] ── Target Symbol Stripping
           │
           ▼
[Standalone Distributable Binary] ── Sub-5MB Native Executable`,
      tradeoffs: [
        { choice: 'OS-Native WebView vs Bundled Chromium', why: 'Saves over 150 MB binary size and 200 MB idle RAM overhead per launched desktop instance.' },
        { choice: 'In-RAM Asset Inlining vs Separate Resource Folders', why: 'Prevents asset tampering and delivers a single, clean executable file for users.' }
      ],
      codeSnippet: {
        language: 'bash',
        title: 'Appify CLI Release Build Invocation',
        code: `# Bundle production single-binary desktop application
$ appify bundle --target x86_64-unknown-linux-gnu --release
[INFO] Bundling frontend distribution from dist/
[INFO] Generating native host wrapper with OS-native WebView
[INFO] Stripping symbols and optimizing asset layout
[INFO] Built standalone binary: target/release/aerovex-workstation (4.2 MB)`
      }
    }
  },
  {
    id: 'custom_protocol_ipc',
    name: 'Custom Protocol Schemes & Zero-Copy IPC',
    domain: 'ipc',
    tierLabel: 'TIER 1 // INTER-PROCESS COMM',
    category: 'IPC & Protocols',
    status: 'ACTIVE',
    statusTone: 'cyan',
    tagline: 'Ultra-fast appify:// custom URI protocol and direct native memory IPC bridge.',
    metric: '<0.12 ms',
    metricDetail: 'Mean Round-Trip IPC Invoke Latency between Webview and Native Rust Core',
    description: 'Bypasses insecure local HTTP/TCP sockets entirely by registering OS custom URI schemes (`appify://`) and using binary serialization for zero-copy memory transfers.',
    highlights: [
      'Zero local open TCP ports, eliminating port collision and network eavesdropping vulnerabilities',
      'Direct binary buffer streaming for high-throughput video frames and telemetry',
      'Type-safe TypeScript bindings generated directly from Rust IPC command definitions'
    ],
    tags: ['appify:// Protocol', 'Zero-Copy IPC', 'Type-Safe Commands', 'No Local Ports'],
    docSectionId: 'custom_protocol_architecture',
    architectureDetails: {
      pipelineDiagram: `Frontend JavaScript / WebAssembly Viewport
           │
           ▼
[Secure Custom Protocol (appify://)] ── Zero Open Local Ports
           │
           ▼
[Native IPC Bridge Dispatcher] ── Low-Latency Message Channel
           │
           ▼
[Hardware / Native Operating System Services] ── Real-Time Response`,
      tradeoffs: [
        { choice: 'Custom Protocol Scheme vs Localhost HTTP', why: 'Eliminates OS firewall warning prompts, port collisions, and cross-origin attack surfaces.' },
        { choice: 'Binary Memory Buffers vs JSON String Encoding', why: 'Accelerates streaming data throughput for high-frequency robotics sensor streams.' }
      ],
      codeSnippet: {
        language: 'typescript',
        title: 'Type-Safe Frontend IPC Invocation',
        code: `import { invoke } from '@appify/api';

interface TelemetryFrame {
    altitude_m: number;
    airspeed_mps: number;
    battery_mv: number;
}

// Invoke native command without opening local network ports
export async function fetchTelemetry(): Promise<TelemetryFrame> {
    return await invoke<TelemetryFrame>('read_hardware_telemetry', {
        bus_id: 1,
        frequency_hz: 100
    });
}`
      }
    }
  },
  {
    id: 'system_tray_window',
    name: 'Native Window & System Tray Controller',
    domain: 'windowing',
    tierLabel: 'TIER 1 // DESKTOP DESIGNS',
    category: 'Native Windowing',
    status: 'ACTIVE',
    statusTone: 'emerald',
    tagline: 'Hardware-accelerated window frames, custom titlebars, and system tray integration.',
    metric: '60 FPS',
    metricDetail: 'Smooth Native Window Resizing & Frameless Titlebar Drag Performance',
    description: 'Provides frameless acrylic/vibrancy window styling, multi-monitor position memory, background daemon minimized tray operation, and global keyboard shortcuts.',
    highlights: [
      'Frameless custom window styling with native OS snapping and shadow support',
      'Persistent multi-monitor geometry state saved across application launches',
      'Zero-CPU background tray daemon mode with instant hotkey summon'
    ],
    tags: ['Frameless Window', 'System Tray', 'Multi-Monitor', 'Global Shortcuts', 'Vibrancy'],
    docSectionId: 'system_tray_architecture',
    architectureDetails: {
      pipelineDiagram: `Operating System Window Manager (Wayland / X11 / Win32 / Cocoa)
           │
           ▼
[Native Window State Controller] ── Frameless Hit-Testing & Multi-Monitor Support
           │
           ▼
[GPU Acrylic / Vibrancy Compositor] ── Clean Surface Blending
           │
           ▼
[System Tray & Global Shortcuts Manager] ── Background Daemon Summoning`,
      tradeoffs: [
        { choice: 'Native Window APIs vs Emulated CSS Windows', why: 'Native window hooks support operating system tiling, snapping, and hardware-accelerated shadows.' },
        { choice: 'Minimized Background Tray Daemon vs Exit', why: 'Enables sub-10ms instant summoning of mission monitoring consoles.' }
      ],
      codeSnippet: {
        language: 'json',
        title: 'Appify Declarative Window Configuration',
        code: `{
  "window": {
    "title": "Aerovex Workstation",
    "width": 1440,
    "height": 900,
    "decorations": false,
    "transparent": true,
    "resizable": true
  },
  "tray": {
    "icon": "icons/tray.png",
    "tooltip": "Aerovex Flight Monitor",
    "minimizeToTray": true
  }
}`
      }
    }
  },
  {
    id: 'auto_update_ed25519',
    name: 'Ed25519 Cryptographic Auto-Updater',
    domain: 'security',
    tierLabel: 'TIER 1 // ENTERPRISE SECURITY',
    category: 'Secure Updates',
    status: 'ACTIVE',
    statusTone: 'amber',
    tagline: 'Differential delta binary updates verified with Ed25519 public key signatures.',
    metric: '100%',
    metricDetail: 'Cryptographically Verified Binary Authenticity & Zero-Downtime Hot Swapping',
    description: 'Safely downloads, verifies, and hot-swaps new application releases using Ed25519 elliptic-curve signatures and differential binary deltas to minimize download bandwidth.',
    highlights: [
      'Ed25519 public key verification preventing man-in-the-middle binary tampering',
      'Differential binary deltas reducing update downloads from 5MB to under 200KB',
      'Atomic replacement preventing corrupted or half-installed states'
    ],
    tags: ['Ed25519 Signatures', 'Differential Updates', 'Atomic Swap', 'Zero Downtime'],
    docSectionId: 'auto_update_architecture',
    architectureDetails: {
      pipelineDiagram: `Remote Secure Release Endpoint
           │
           ▼
[Ed25519 Cryptographic Verification Gate] ── Signature Authenticity Check
           │
           ▼
[Differential Binary Delta Reconstructor] ── Sub-200KB Payload
           │
           ▼
[Atomic Application Hot-Swap] ── Safe Overwrite with Fallback Protection`,
      tradeoffs: [
        { choice: 'Ed25519 Public Key Gate vs Standard HTTPS Alone', why: 'Protects critical field devices against CDN cache poisoning and untrusted network tampering.' },
        { choice: 'Differential Delta Patches vs Full Package', why: 'Reduces update bandwidth by over 95% for reliable updates across cellular and satellite connections.' }
      ],
      codeSnippet: {
        language: 'typescript',
        title: 'Appify Client Auto-Update Verification API',
        code: `import { checkUpdate } from '@appify/updater';

// Check for verified releases with Ed25519 signature
const update = await checkUpdate();
if (update.available) {
    console.log(\`Verified release v\${update.version} found (\${update.sizeKb} KB delta).\`);
    await update.downloadAndInstall();
    await update.relaunch();
}`
      }
    }
  },
  {
    id: 'security_sandbox',
    name: 'Isolated Webview Security Sandbox',
    domain: 'isolation',
    tierLabel: 'TIER 1 // MEMORY SAFETY',
    category: 'Sandbox & Permissions',
    status: 'OPTIMIZED',
    statusTone: 'indigo',
    tagline: 'Strict CSP enforcement, fine-grained capability scopes, and zero remote code execution.',
    metric: '0 RCE',
    metricDetail: 'Zero Remote Code Execution Vulnerabilities & Complete Isolation of Native APIs',
    description: 'Enforces strict Content Security Policies (CSP), fine-grained capability permission matrices, and complete isolation of native OS commands from untrusted web content.',
    highlights: [
      'Granular capability scopes restricting filesystem access to explicitly permitted directories',
      'Automatic CSP injection preventing unauthorized cross-origin script execution',
      'No nodeIntegration or dangerous Chromium flags enabled'
    ],
    tags: ['Security Sandbox', 'Capability Scopes', 'Strict CSP', 'Zero RCE', 'Sandboxed IPC'],
    docSectionId: 'security_sandbox_architecture',
    architectureDetails: {
      pipelineDiagram: `Client Web Application Viewport
           │
           ▼
[Content Security Policy Engine] ── Script & Resource Origin Isolation
           │
           ▼
[Granular Capabilities ACL Scope] ── Explicit Permission Rules
           │
           ▼
[Protected Host Hardware Interface] ── Zero Unrestricted System Access`,
      tradeoffs: [
        { choice: 'Explicit Capability ACLs vs Open FFI Calls', why: 'Guarantees third-party web plugins cannot access arbitrary filesystem directories or network sockets.' },
        { choice: 'Rust Memory-Safe Backend vs Node Addons', why: 'Eliminates memory safety bugs and buffer overflow attack vectors in native plugins.' }
      ],
      codeSnippet: {
        language: 'json',
        title: 'Appify Security Capabilities Scope Definition',
        code: `{
  "$schema": "https://appify.aerovex.net/schema/capabilities.json",
  "identifier": "workstation-runtime-scope",
  "description": "Least-privilege permissions for Workstation UI",
  "permissions": [
    "core:default",
    {
      "identifier": "fs:allow-read-file",
      "allow": [{ "path": "$APP_DATA/projects/**" }]
    }
  ]
}`
      }
    }
  }
];
