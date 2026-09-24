/**
 * Appify Static Marketplace CDN Edge Worker
 * Deployed to cdn.appify.aerovex.net via Cloudflare Workers
 * 
 * Features:
 * - Ultra-fast global edge asset streaming
 * - Zero background server cost (100% serverless/static assets)
 * - Automatic CORS headers for cross-origin webview/Tauri access
 * - Smart caching headers (5 min browser cache, 1 hour edge CDN cache)
 */

export default {
  async fetch(request, env) {
    // Handle CORS preflight requests
    if (request.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "GET, HEAD, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type, Range, If-None-Match",
          "Access-Control-Max-Age": "86400",
        },
      });
    }

    if (request.method !== "GET" && request.method !== "HEAD") {
      return new Response("Method not allowed", { status: 405 });
    }

    try {
      // Fetch static asset from Worker Assets binding
      const response = await env.ASSETS.fetch(request);

      // Clone and attach CORS and caching headers
      const headers = new Headers(response.headers);
      headers.set("Access-Control-Allow-Origin", "*");
      headers.set("X-Content-Type-Options", "nosniff");
      
      const url = new URL(request.url);
      if (url.pathname.endsWith(".json")) {
        headers.set("Content-Type", "application/json; charset=utf-8");
        headers.set("Cache-Control", "public, max-age=180, s-maxage=900, stale-while-revalidate=86400");
      } else if (url.pathname.endsWith(".js")) {
        headers.set("Content-Type", "application/javascript; charset=utf-8");
        headers.set("Cache-Control", "public, max-age=86400, s-maxage=604800, immutable");
      } else if (url.pathname.endsWith(".css")) {
        headers.set("Content-Type", "text/css; charset=utf-8");
        headers.set("Cache-Control", "public, max-age=86400, s-maxage=604800, immutable");
      }

      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers,
      });
    } catch (err) {
      return new Response(JSON.stringify({ error: "CDN Error", message: String(err) }), {
        status: 500,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      });
    }
  },
};
