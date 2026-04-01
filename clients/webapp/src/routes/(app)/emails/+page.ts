import { apiFetch } from "$lib/mobile/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ data, url }) => {
    if (data && "emails" in data) return data;

    const p = url.searchParams.get("p") ?? "1";
    const result = await apiFetch<{ 
        emails: unknown[]; 
        total: number; 
        pageSize: number
        errror?: string; 
    }>(
        `/api/emails?p=${p}`,
    );
    if (!result.ok) return { emails: [], total: 0, pageSize: 10, error: result.error };
    return result.data;
};
