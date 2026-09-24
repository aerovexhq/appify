# Appify Community Marketplace CDN

This directory contains the static edge worker and catalog builder for **`cdn.appify.aerovex.net`**, hosted on Cloudflare Workers using Static Assets.

## Architecture

- **Domain**: `cdn.appify.aerovex.net`
- **Zone**: `aerovex.net` (Cloudflare)
- **Infrastructure**: Cloudflare Workers Pure Static Assets (`wrangler.jsonc`)
- **Cost**: **$0.00 / Zero persistent servers / Zero compute workers** (100% static assets served directly from Cloudflare global edge CDN)
- **CORS & Cache**: Configured statically via `public/_headers` (`Access-Control-Allow-Origin: *`, immutable long-lived asset caching, and fresh catalog caching)

## Directory Layout

```text
marketplace/
├── extensions/              # Source code of community extensions
│   ├── plugins/
│   │   └── <id>/
│   │       ├── manifest.json
│   │       └── code.js
│   └── themes/
│       └── <id>/
│           ├── manifest.json
│           └── style.css
├── public/                  # Auto-generated static files (uploaded directly to Cloudflare)
│   ├── _headers             # Static edge CORS and caching rules
│   ├── index.json
│   └── v1/
│       ├── marketplace.json # Master catalog of all extensions
│       ├── plugins/         # Raw .js files and individual metadata .json
│       └── themes/          # Raw .css files and individual metadata .json
├── scripts/
│   └── build-marketplace.js # Compiles manifests, checks SHA256 hashes, generates _headers and static files
├── package.json
└── wrangler.jsonc           # Pure static Cloudflare configuration (no worker script)
```

## How to Add an Extension from a GitHub Issue

When a community contributor submits an issue on GitHub using the Plugin or Theme template:

1. Create a folder in `marketplace/extensions/plugins/<id>` (or `themes/<id>`).
2. Create `manifest.json`:
   ```json
   {
     "id": "my-extension",
     "name": "My Extension",
     "description": "What it does",
     "category": "plugin",
     "author": "Contributor",
     "author_url": "https://github.com/contributor",
     "version": "1.0.0",
     "target_domains": ["example.com"],
     "tags": ["tag1", "tag2"]
   }
   ```
3. Place `code.js` (for plugins) or `style.css` (for themes).
4. Run the 1-command build and deploy:
   ```bash
   cd marketplace
   npm run deploy
   ```
   *(Or `node scripts/build-marketplace.js && npx wrangler deploy`)*

Cloudflare will immediately publish the updated index and assets globally within seconds!
