import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Appify',
  description: 'Transform Any Web App Into A Native Desktop App',
  head: [
    ['link', { rel: 'icon', type: 'image/png', href: '/favicon.png' }],
    ['meta', { name: 'theme-color', content: '#3b82f6' }]
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
      { text: 'GitHub', link: 'https://github.com/aerovexsim/appify' }
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
      { icon: 'github', link: 'https://github.com/aerovexsim/appify' }
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
