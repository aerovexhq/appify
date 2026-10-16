import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Appify',
  description: 'Transform Any Web App Into A Native Desktop App',
  cleanUrls: true,
  appearance: 'dark',
  head: [
    ['link', { rel: 'icon', type: 'image/png', href: '/favicon.png' }],
    ['meta', { name: 'theme-color', content: '#08090a' }],
    ['meta', { name: 'color-scheme', content: 'dark' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Appify — Transform Any Web App Into A Native Desktop App' }],
    ['meta', { property: 'og:description', content: 'Ultra-fast, isolated, lightweight native desktop apps powered by Tauri & WebKitGTK.' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { href: 'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600;700&display=swap', rel: 'stylesheet' }]
  ],
  themeConfig: {
    logo: '/favicon.png',
    siteTitle: 'Appify',
    nav: [
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'Presets', link: '/guide/presets' },
      { text: 'Extensions', link: '/guide/extensions' },
      { text: 'Marketplace', link: '/guide/marketplace' },
      { text: 'CLI Reference', link: '/guide/cli' },
      {
        text: 'Aerovex Ecosystem',
        items: [
          { text: 'Aerovex Platform', link: 'https://aerovex.net', target: '_self' },
          { text: 'Axiom EDA', link: 'https://axiom.aerovex.net', target: '_self' },
          { text: 'Phonon CAD', link: 'https://phonon.aerovex.net', target: '_self' },
          { text: 'Atoma CAD', link: 'https://atoma.aerovex.net', target: '_self' },
          { text: 'Sonon DSP', link: 'https://sonon.aerovex.net', target: '_self' },
          { text: 'Vexel Lang', link: 'https://vexel.aerovex.net', target: '_self' },
          { text: 'Appify', link: 'https://appify.aerovex.net', target: '_self' }
        ]
      },
      { text: 'GitHub', link: 'https://github.com/aerovexhq/appify' }
    ],
    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Introduction', link: '/guide/introduction' },
          { text: 'Installation', link: '/guide/installation' }
        ]
      },
      {
        text: 'Core Features',
        items: [
          { text: 'Curated Presets', link: '/guide/presets' },
          { text: 'Plugins & Themes Studio', link: '/guide/extensions' },
          { text: 'Community Marketplace', link: '/guide/marketplace' }
        ]
      },
      {
        text: 'Reference',
        items: [
          { text: 'CLI Cheat Sheet', link: '/guide/cli' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/aerovexhq/appify' }
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026 Aerovex & Appify Contributors'
    },
    search: {
      provider: 'local'
    }
  }
})
