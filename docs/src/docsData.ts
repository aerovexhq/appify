export interface DocSectionContent {
  title: string;
  badge: string;
  description: string;
  configJson?: string;
  rustCode?: string;
  tsCode?: string;
  callout?: {
    type: 'note' | 'tip' | 'warning' | 'spec';
    title: string;
    text: string;
  };
  specs?: { param: string; val: string; note: string }[];
  steps?: { title: string; desc: string; cmd?: string }[];
}

export interface DocSection {
  id: string;
  title: string;
  badge: string;
  content: DocSectionContent;
}

export interface DocCategory {
  id: string;
  title: string;
  badge: string;
  sections: DocSection[];
}

export const APPIFY_DOC_TREE: DocCategory[] = [
  {
    id: 'getting_started',
    title: '1. Getting Started',
    badge: 'CORE',
    sections: [
      {
        id: 'why_appify',
        title: 'Why Appify',
        badge: 'OVERVIEW',
        content: {
          title: 'Modern Web-to-Native Desktop Application Bundler on Tauri 2.0',
          badge: 'ARCHITECTURE',
          description: 'Appify transforms any modern web application (React, Vue, Svelte, Vite, WASM) into an ultra-compact, secure, single-binary native desktop application using Tauri 2.0. By leveraging OS-native WebViews, Appify produces binaries under 5MB with sub-24MB idle memory footprints and zero Chromium dependencies.',
          callout: {
            type: 'spec',
            title: 'Zero Chromium Overhead',
            text: 'Appify does not bundle Chromium or Node.js runtimes. It binds directly to the operating system WebKitGTK (Linux), WKWebView (macOS), or WebView2 (Windows), reducing bundle size by 97%.'
          },
          specs: [
            { param: 'Distributable Binary Size', val: '<4.8 MB', note: 'Standalone executable with inlined assets' },
            { param: 'Idle RAM Consumption', val: '<24 MB', note: 'Compared to 200MB+ for Electron' },
            { param: 'Cold Launch Latency', val: '<180 ms', note: 'Instant native window initialization' },
            { param: 'Differential Updates', val: '<200 KB', note: 'Ed25519 signed byte-level delta patches' }
          ],
          configJson: `// appify.config.json: Desktop Application Configuration
{
  "$schema": "https://appify.aerovex.net/schema/v2.json",
  "productName": "Aerovex Workstation",
  "version": "2.4.0",
  "identifier": "net.aerovex.workstation",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5180",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Aerovex Workstation",
        "width": 1440,
        "height": 900,
        "decorations": false,
        "transparent": true
      }
    ],
    "security": {
      "csp": "default-src 'self' appify:; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'"
    }
  }
}`,
          rustCode: `// Rust Native IPC Handler Registration
use tauri::command;

#[command]
pub fn execute_native_telemetry(channel_id: u32) -> Result<String, String> {
    // Pure safe Rust hardware access
    Ok(format!("Telemetry channel {} active at 100 Hz", channel_id))
}`,
          tsCode: `// TypeScript Client IPC Invocation
import { invoke } from '@appify/api';

const status = await invoke<string>('execute_native_telemetry', { channel_id: 1 });
console.log('Native response:', status);`
        }
      },
      {
        id: 'installation',
        title: 'Installation & Toolchain',
        badge: 'SETUP',
        content: {
          title: 'Installing the Appify Bundler Toolchain',
          badge: 'ENVIRONMENT',
          description: 'Install the Appify CLI globally via cargo or integrate the npm package into your frontend build pipelines.',
          steps: [
            {
              title: 'Install Appify CLI via Cargo',
              desc: 'Install the latest high-performance native bundler binary.',
              cmd: 'cargo install appify-cli --locked'
            },
            {
              title: 'Initialize in Existing Project',
              desc: 'Configure Tauri 2.0 native bindings in an existing Vite or React workspace.',
              cmd: 'appify init --template react-ts'
            },
            {
              title: 'Launch Development Mode',
              desc: 'Start the native desktop host with hot-module replacement (HMR).',
              cmd: 'appify dev'
            }
          ]
        }
      }
    ]
  },
  {
    id: 'bundling',
    title: '2. Bundling & Distribution',
    badge: 'PACKAGING',
    sections: [
      {
        id: 'single_binary_architecture',
        title: 'Single-Binary Multi-Platform',
        badge: 'CROSS-PLATFORM',
        content: {
          title: 'Compiling Single-Binary Standalone Executables',
          badge: 'DISTRIBUTION',
          description: 'Appify embeds compiled web distribution folders directly into the native Rust binary via compile-time static inlining and zero-copy in-RAM decompression.',
          specs: [
            { param: 'Linux Output', val: 'Standalone ELF binary & AppImage', note: 'Gluster/musl compatible' },
            { param: 'macOS Output', val: 'Universal Binary (.app / .dmg)', note: 'x86_64 & Apple Silicon ARM64' },
            { param: 'Windows Output', val: 'Single-file .exe & MSI installer', note: 'Signed with Authenticode' }
          ],
          callout: {
            type: 'tip',
            title: 'Cross-Compilation with Zig',
            text: 'Appify integrates the Zig C/C++ toolchain to allow building Windows and macOS targets directly from Linux continuous integration runners.'
          }
        }
      }
    ]
  },
  {
    id: 'ipc_protocols',
    title: '3. IPC & Custom Protocols',
    badge: 'IPC',
    sections: [
      {
        id: 'custom_protocol_architecture',
        title: 'Custom Protocol Schemes (appify://)',
        badge: 'SECURITY',
        content: {
          title: 'Zero-Port Custom Protocol Architecture',
          badge: 'NETWORKING',
          description: 'Serves web assets and handles IPC over custom OS URI schemes (appify://), completely bypassing local TCP port binding and protecting against localhost port scanning.',
          specs: [
            { param: 'Local Ports Opened', val: '0 (Zero)', note: 'No localhost HTTP sockets created' },
            { param: 'Round-Trip IPC Latency', val: '<0.12 ms', note: 'Direct native OS window message pump' },
            { param: 'Type Safety', val: 'End-to-End TypeScript', note: 'Generated from Rust command signatures' }
          ]
        }
      }
    ]
  },
  {
    id: 'auto_updates',
    title: '4. Auto-Updates & Security',
    badge: 'SECURITY',
    sections: [
      {
        id: 'auto_update_architecture',
        title: 'Ed25519 Cryptographic Updates',
        badge: 'ED25519',
        content: {
          title: 'Differential Delta Updates & Atomic Binary Swapping',
          badge: 'UPDATES',
          description: 'Updates desktop installations over cellular or low-bandwidth links by calculating binary byte-level differential deltas and verifying Ed25519 cryptographic signatures.',
          specs: [
            { param: 'Signature Scheme', val: 'Ed25519 Elliptic Curve', note: 'Deterministic 64-byte signatures' },
            { param: 'Mean Delta Size', val: '<200 KB', note: 'Only modified binary bytes transmitted' },
            { param: 'Rollback Protection', val: 'Atomic Swap with Fallback', note: 'Corrupted downloads never executed' }
          ]
        }
      },
      {
        id: 'security_sandbox_architecture',
        title: 'Capabilities & Sandbox Matrix',
        badge: 'SANDBOX',
        content: {
          title: 'Fine-Grained Capability Access Control Lists (ACL)',
          badge: 'PERMISSIONS',
          description: 'Limits frontend JavaScript access to explicitly allowed filesystem paths, native commands, and OS capabilities using Tauri 2.0 security scopes.',
          specs: [
            { param: 'Filesystem Scope', val: 'Restricted to $APP_DATA by default', note: 'No root directory access' },
            { param: 'CSP Enforcement', val: 'Automatic Nonce Injection', note: 'Blocks inline eval and unauthorized CDNs' },
            { param: 'RCE Vulnerabilities', val: '0 (Zero)', note: 'No Node.js runtime exposed to web context' }
          ]
        }
      }
    ]
  }
];
