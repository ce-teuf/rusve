import { json } from "@sveltejs/kit";
import { grpcSafe } from "$lib/safe";
import { usersService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import type { AuthResponse__Output } from "$lib/proto/proto/AuthResponse";
import type { Safe } from "$lib/safe";

export type ApiUser = {
    id: string;
    email: string;
    subscription_active: boolean;
};

/**
 * Validates the Bearer token from Authorization header.
 * Returns the authenticated user or a 401 JSON response.
 */
export async function apiAuth(
    request: Request,
): Promise<{ user: ApiUser } | Response> {
    const authHeader = request.headers.get("Authorization");
    const token = authHeader?.replace("Bearer ", "").trim() ?? "";

    if (!token) {
        return json({ error: "Unauthorized" }, { status: 401 });
    }

    const metadata = await createMetadata(token);
    const auth = await new Promise<Safe<AuthResponse__Output>>((res) => {
        usersService.Auth({}, metadata, grpcSafe(res));
    });

    if (auth.error || !auth.data.user) {
        return json({ error: "Unauthorized" }, { status: 401 });
    }

    return {
        user: {
            id: auth.data.user.id,
            email: auth.data.user.email,
            subscription_active: auth.data.user.subscription_active,
        },
    };
}
