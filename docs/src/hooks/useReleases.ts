import { useState, useEffect } from 'react';

export interface ReleaseAsset {
  name: string;
  size: number;
  download_count: number;
  browser_download_url: string;
  updated_at: string;
  content_type: string;
}

export interface Release {
  tag_name: string;
  name: string;
  published_at: string;
  html_url: string;
  body: string;
  prerelease: boolean;
  assets: ReleaseAsset[];
}

export function useReleases(repo: string, defaultReleases: Release[] = []) {
  const [releases, setReleases] = useState<Release[]>(defaultReleases);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchRuntime() {
      try {
        setLoading(true);
        let res = await fetch(`https://api.github.com/repos/${repo}/releases`);
        let data: any = null;
        if (res.ok) {
          data = await res.json();
        }

        if (!data || !Array.isArray(data) || data.length === 0) {
          res = await fetch(`https://api.github.com/repos/${repo}/releases/latest`);
          if (res.ok) {
            const latest = await res.json();
            data = [latest];
          }
        }

        if (!cancelled && data && Array.isArray(data) && data.length > 0) {
          const formatted: Release[] = data.map((rel: any) => ({
            tag_name: rel.tag_name,
            name: rel.name || rel.tag_name,
            published_at: rel.published_at || rel.created_at,
            html_url: rel.html_url,
            body: rel.body || '',
            prerelease: rel.prerelease || false,
            assets: (rel.assets || []).map((a: any) => ({
              name: a.name,
              size: a.size,
              download_count: a.download_count,
              browser_download_url: a.browser_download_url,
              updated_at: a.updated_at,
              content_type: a.content_type,
            })),
          }));
          setReleases(formatted);
          setError(null);
        }
      } catch (err: any) {
        if (!cancelled) {
          setError(err.message || 'Failed to fetch releases dynamically');
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    fetchRuntime();
    return () => {
      cancelled = true;
    };
  }, [repo]);

  const latestRelease = releases.length > 0 ? releases[0] : null;

  return { releases, latestRelease, loading, error };
}
