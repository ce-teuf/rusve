import { json } from "@sveltejs/kit";
import { grpcSafe } from "$lib/safe";
import { usersService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import type { RequestHandler } from "./$types";

/**
 * POST /api/subscription
 * Body: { action: "checkout" | "portal" }
 * Returns: { url: string } — the Stripe redirect URL (mobile opens it in browser)
 */
export const POST: RequestHandler = async ({ request }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const body = await request.json().catch(() => ({}));
    const action = String(body.action ?? "checkout");

    const metadata = await createMetadata(auth.user.id);

    if (action === "portal") {
        const s = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/StripeUrlResponse").StripeUrlResponse__Output>>((r) =>
            usersService.CreateStripePortal({}, metadata, grpcSafe(r)),
        );
        if (s.error) return json({ error: s.msg }, { status: 500 });
        return json({ url: s.data.url });
    }

    const s = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/StripeUrlResponse").StripeUrlResponse__Output>>((r) =>
        usersService.CreateStripeCheckout({}, metadata, grpcSafe(r)),
    );
    if (s.error) return json({ error: s.msg }, { status: 500 });
    return json({ url: s.data.url });
};
