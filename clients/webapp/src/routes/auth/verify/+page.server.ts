import { COOKIE_DOMAIN } from "$env/static/private";
import { PUBLIC_AUTH_URL } from "$env/static/public";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ url }) => {
    const email = url.searchParams.get("email") ?? "";
    return { email };
};

export const actions: Actions = {
    resend: async ({ request }) => {
        const data = await request.formData();
        const email = data.get("email")?.toString().trim() ?? "";
        if (!email) return fail(400, { email, error: "Email is required" });

        let res: Response;
        try {
            res = await fetch(`${PUBLIC_AUTH_URL}/resend-verification`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ email }),
            });
        } catch {
            return fail(503, { email, error: "Auth service unreachable" });
        }

        if (!res.ok) {
            const body = await res.json().catch(() => ({})) as { error?: string };
            return fail(400, { email, error: body.error ?? "Failed to resend code" });
        }

        return { email, message: "Verification code resent" };
    },
    verify: async ({ request, cookies, url }) => {
        const data = await request.formData();
        const email = data.get("email")?.toString().trim() ?? "";
        const code = data.get("code")?.toString().trim().toUpperCase() ?? "";

        if (!email || !code) {
            return fail(400, { email, error: "Email and code are required" });
        }

        if (code.length !== 6) {
            return fail(400, { email, error: "Code must be 6 characters" });
        }

        if (!/^[A-Z1-9]{6}$/.test(code)) {
            return fail(400, { email, error: "Code must contain only letters A-Z and numbers 1-9" });
        }

        let res: Response;
        try {
            res = await fetch(`${PUBLIC_AUTH_URL}/verify`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ email, code }),
            });
        } catch {
            return fail(503, { email, error: "Auth service unreachable" });
        }

        if (!res.ok) {
            const body = await res.json().catch(() => ({})) as { error?: string };
            return fail(400, { email, error: body.error ?? "Verification failed" });
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