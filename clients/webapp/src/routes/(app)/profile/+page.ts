import { apiFetch } from "$lib/mobile/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ data }) => {
    if (data && "profile" in data) return data;

    const result = await apiFetch<{ profile: unknown; resume: unknown }>("/api/profile");
    if (!result.ok) return { profile: null, resume: undefined, error: result.error };
    return result.data;
};
