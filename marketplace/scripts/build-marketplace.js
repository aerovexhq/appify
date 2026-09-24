#!/usr/bin/env node

import fs from "fs";
import path from "path";
import crypto from "crypto";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const EXTENSIONS_DIR = path.join(ROOT, "extensions");
const PUBLIC_DIR = path.join(ROOT, "public");
const V1_DIR = path.join(PUBLIC_DIR, "v1");
const CDN_BASE_URL = "https://cdn.appify.aerovex.net";

console.log("🔨 Building Appify Community Marketplace CDN index...");

// Ensure output directories exist
fs.mkdirSync(path.join(V1_DIR, "plugins"), { recursive: true });
fs.mkdirSync(path.join(V1_DIR, "themes"), { recursive: true });

const sha256 = (content) => crypto.createHash("sha256").update(content, "utf8").digest("hex");

const pluginsDir = path.join(EXTENSIONS_DIR, "plugins");
const themesDir = path.join(EXTENSIONS_DIR, "themes");

const items = [];

// Helper to scan a directory for extension packages
const processCategory = (categoryDir, category) => {
  if (!fs.existsSync(categoryDir)) return;

  const entries = fs.readdirSync(categoryDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;

    const extPath = path.join(categoryDir, entry.name);
    const manifestPath = path.join(extPath, "manifest.json");

    if (!fs.existsSync(manifestPath)) {
      console.warn(`⚠️ Skipping ${entry.name}: missing manifest.json`);
      continue;
    }

    try {
      const rawManifest = fs.readFileSync(manifestPath, "utf8");
      const manifest = JSON.parse(rawManifest);

      const isPlugin = category === "plugin";
      const codeFileName = isPlugin ? "code.js" : "style.css";
      const codePath = path.join(extPath, codeFileName);

      if (!fs.existsSync(codePath)) {
        console.warn(`⚠️ Skipping ${entry.name}: missing ${codeFileName}`);
        continue;
      }

      const rawCode = fs.readFileSync(codePath, "utf8");
      const hash = sha256(rawCode);
      const subfolder = isPlugin ? "plugins" : "themes";
      const fileExt = isPlugin ? ".js" : ".css";
      const destFileName = `${manifest.id}${fileExt}`;
      const destPath = path.join(V1_DIR, subfolder, destFileName);

      // Write raw file to public directory
      fs.writeFileSync(destPath, rawCode, "utf8");

      const itemEntry = {
        id: manifest.id,
        name: manifest.name,
        description: manifest.description || "",
        category: manifest.category || category,
        author: manifest.author || "Community",
        author_url: manifest.author_url || null,
        version: manifest.version || "1.0.0",
        target_domains: manifest.target_domains || ["*"],
        tags: manifest.tags || [],
        icon: manifest.icon || null,
        sha256: hash,
        size_bytes: Buffer.byteLength(rawCode, "utf8"),
        download_url: `${CDN_BASE_URL}/v1/${subfolder}/${destFileName}`,
        // Include direct inline code in catalog for fast preview
        preview_code: rawCode.slice(0, 1500),
        code_content: rawCode,
        created_at: manifest.created_at || new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };

      // Write individual item JSON
      const itemJsonPath = path.join(V1_DIR, subfolder, `${manifest.id}.json`);
      fs.writeFileSync(itemJsonPath, JSON.stringify(itemEntry, null, 2), "utf8");

      items.push(itemEntry);
      console.log(`  ✓ Processed [${category.toUpperCase()}] ${manifest.name} (${manifest.id})`);
    } catch (err) {
      console.error(`❌ Error processing ${entry.name}:`, err.message);
    }
  }
};

processCategory(pluginsDir, "plugin");
processCategory(themesDir, "theme");

// Sort items by name
items.sort((a, b) => a.name.localeCompare(b.name));

const catalog = {
  version: "1.0.0",
  cdn_base: CDN_BASE_URL,
  generated_at: new Date().toISOString(),
  total_items: items.length,
  plugins_count: items.filter((i) => i.category === "plugin").length,
  themes_count: items.filter((i) => i.category === "theme").length,
  items,
};

// Write catalog files
const catalogPath = path.join(V1_DIR, "marketplace.json");
const indexPath = path.join(V1_DIR, "index.json");
fs.writeFileSync(catalogPath, JSON.stringify(catalog, null, 2), "utf8");
fs.writeFileSync(indexPath, JSON.stringify(catalog, null, 2), "utf8");

// Also write a root index.json
fs.writeFileSync(path.join(PUBLIC_DIR, "index.json"), JSON.stringify(catalog, null, 2), "utf8");

console.log(`\n🎉 Successfully built marketplace catalog with ${items.length} items!`);
console.log(`   Output: ${catalogPath}`);
