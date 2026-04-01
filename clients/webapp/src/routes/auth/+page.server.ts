import { COOKIE_DOMAIN } from "$env/static/private";
import { PUBLIC_AUTH_URL } from "$env/static/public";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";

export const actions: Actions = {
    login: async ({ request, cookies }) => {
        const data = await request.formData();
        const email = data.get("email")?.toString().trim() ?? "";
        const password = data.get("password")?.toString() ?? "";

        if (!email || !password) {
            return fail(400, { email, error: "Email and password are required" });
        }

        let res: Response;
        try {
            res = await fetch(`${PUBLIC_AUTH_URL}/local-login`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ email, password }),
            });
        } catch {
            return fail(503, { email, error: "Auth service unreachable" });
        }

        if (!res.ok) {
            const body = await res.json().catch(() => ({})) as { error?: string };
            return fail(400, { email, error: body.error ?? "Login failed" });
        }

        const { token } = await res.json() as { token: string };
        cookies.set("token", token, {
            domain: COOKIE_DOMAIN,
            path: "/",
            maxAge: 604800,
            httpOnly: true,
            sameSite: "lax",
            secure: true,
        });
        redirect(302, "/dashboard");
    },
};
