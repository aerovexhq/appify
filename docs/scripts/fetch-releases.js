import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const REPO = process.env.VITE_GITHUB_REPO || 'aerovexhq/appify';
const OUT_FILE = path.resolve(__dirname, '../src/data/releases.json');

async function fetchReleases() {
  console.log(`[fetch-releases] Checking GitHub releases for ${REPO}...`);
  const headers = {
    'User-Agent': 'aerovex-docs-builder',
    'Accept': 'application/vnd.github.v3+json',
  };
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  try {
    let res = await fetch(`https://api.github.com/repos/${REPO}/releases`, { headers });
    let data = null;
    if (res.ok) {
      data = await res.json();
    }

    if (!data || !Array.isArray(data) || data.length === 0) {
      console.log(`[fetch-releases] /releases returned empty; querying /releases/latest...`);
      res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers });
      if (res.ok) {
        const latest = await res.json();
        data = [latest];
      }
    }

    if (data && Array.isArray(data) && data.length > 0) {
      const formatted = data.map(rel => ({
        tag_name: rel.tag_name,
        name: rel.name || rel.tag_name,
        published_at: rel.published_at || rel.created_at,
        html_url: rel.html_url,
        body: rel.body || '',
        prerelease: rel.prerelease || false,
        assets: (rel.assets || []).map(a => ({
          name: a.name,
          size: a.size,
          download_count: a.download_count,
          browser_download_url: a.browser_download_url,
          updated_at: a.updated_at,
          content_type: a.content_type,
        }))
      }));

      fs.mkdirSync(path.dirname(OUT_FILE), { recursive: true });
      fs.writeFileSync(OUT_FILE, JSON.stringify(formatted, null, 2));
      console.log(`[fetch-releases] Successfully saved ${formatted.length} release(s) to ${OUT_FILE}`);
      return;
    }
  } catch (err) {
    console.warn(`[fetch-releases] Network/API warning: ${err.message}`);
  }

  if (fs.existsSync(OUT_FILE)) {
    console.log(`[fetch-releases] Preserving existing ${OUT_FILE}`);
    return;
  }

  console.log(`[fetch-releases] Creating initial fallback releases.json...`);
  fs.mkdirSync(path.dirname(OUT_FILE), { recursive: true });
  fs.writeFileSync(OUT_FILE, JSON.stringify([], null, 2));
}

fetchReleases();
