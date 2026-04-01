import { apiFetch } from "$lib/mobile/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ data, url }) => {
    if (data && "files" in data) return data;

    const p = url.searchParams.get("p") ?? "1";
    const result = await apiFetch<{ files: unknown[]; total: number; pageSize: number }>(
        `/api/files?p=${p}`,
    );
    if (!result.ok) return { files: [], total: 0, pageSize: 10, error: result.error };
    return result.data;
};
