/**
 * Universal load — runs on server (SSR/web) AND browser (mobile SPA).
 *
 * On web: +page.server.ts already ran and `data` contains notes/total/pageSize.
 *         We just pass it through (no duplicate request).
 * On mobile: no server runs, `data` is empty → we fetch from REST API.
 */
import { apiFetch } from "$lib/mobile/api";
import type { NoteResponse__Output } from "$lib/proto/proto/NoteResponse";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ data, url }) => {
    // Web SSR: +page.server.ts populated data — pass through
    if (data && "notes" in data) return data;

    // Mobile SPA: fetch from REST API
    const p = url.searchParams.get("p") ?? "1";
    const result = await apiFetch<{ 
        notes: NoteResponse__Output[]; 
        total: number; 
        pageSize: number;
        error: string;
    }>(
        `/api/notes?p=${p}`,
    );
    if (!result.ok) return { notes: [], total: 0, pageSize: 10, error: result.error };
    return result.data;
};
