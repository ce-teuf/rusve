import { COOKIE_DOMAIN } from "$env/static/private";
import { PUBLIC_AUTH_URL } from "$env/static/public";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";

function validatePassword(password: string): string | null {
    if (password.length < 12) return "Password must be at least 12 characters";
    const digitCount = (password.match(/\d/g) || []).length;
    if (digitCount < 2) return "Password must contain at least 2 digits";
    if (!/[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(password)) return "Password must contain at least 1 special character";
    if (!/[A-Z]/.test(password)) return "Password must contain at least 1 uppercase letter";
    if (!/[a-z]/.test(password)) return "Password must contain at least 1 lowercase letter";
    return null;
}

export const actions: Actions = {
    default: async ({ request, cookies }) => {
        const data = await request.formData();
        const email = data.get("email")?.toString().trim() ?? "";
        const password = data.get("password")?.toString() ?? "";
        const confirm = data.get("confirm_password")?.toString() ?? "";

        if (!email || !password || !confirm) {
            return fail(400, { email, error: "All fields are required" });
        }
        if (password !== confirm) {
            return fail(400, { email, error: "Passwords do not match" });
        }

        const passwordError = validatePassword(password);
        if (passwordError) {
            return fail(400, { email, error: passwordError });
        }

        let res: Response;
        try {
            res = await fetch(`${PUBLIC_AUTH_URL}/local-register`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ email, password }),
            });
        } catch {
            return fail(503, { email, error: "Auth service unreachable" });
        }

        if (!res.ok) {
            const body = await res.json().catch(() => ({})) as { error?: string };
            return fail(400, { email, error: body.error ?? "Registration failed" });
        }

        const { message } = await res.json() as { message: string };
        redirect(302, `/auth/verify?email=${encodeURIComponent(email)}`);
    },
};