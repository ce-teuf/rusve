/**
 * apiFetch — wraps fetch() with the Authorization header for mobile REST API calls.
 *
 * Usage:
 *   const data = await apiFetch<{ notes: Note[] }>('/api/notes');
 *   const data = await apiFetch<{ note: Note }>('/api/notes', {
 *       method: 'POST',
 *       body: JSON.stringify({ title, content }),
 *   });
 */

import { getToken } from "$lib/mobile/auth";

export type ApiResult<T> =
    | { ok: true; data: T }
    | { ok: false; error: string; fields?: { field: string; tag: string }[] };

export async function apiFetch<T>(
    path: string,
    init: RequestInit = {},
): Promise<ApiResult<T>> {
    try {
        const token = await getToken();

        const headers = new Headers(init.headers);
        if (token) headers.set("Authorization", `Bearer ${token}`);
        if (!headers.has("Content-Type") && !(init.body instanceof FormData)) {
            headers.set("Content-Type", "application/json");
        }

        const response = await fetch(path, { ...init, headers });

        if (!response.ok) {
            const body = await response.json().catch(() => ({}));
            return {
                ok: false,
                error: body.error ?? `HTTP ${response.status}`,
                fields: body.fields,
            };
        }

        const data = (await response.json()) as T;
        return { ok: true, data };
    } catch (err) {
        return { ok: false, error: err instanceof Error ? err.message : "Network error" };
    }
}
