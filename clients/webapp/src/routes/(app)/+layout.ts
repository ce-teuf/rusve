/**
 * Universal layout load for the (app) group.
 *
 * On web: +layout.server.ts provides email/avatar from server session.
 * On mobile: no server → return empty strings (Nav will handle gracefully).
 */
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = ({ data }) => {
    if (data && "email" in data) return data;
    return { email: "", avatar: "" };
};
