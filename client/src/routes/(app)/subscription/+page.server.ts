import { grpcSafe } from "$lib/safe";
import { usersService } from "$lib/server/grpc";
import { perf } from "$lib/server/logger";
import { createMetadata } from "$lib/server/metadata";
import { fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";

export const load: PageServerLoad = ({ locals }) => {
    return { subscriptionActive: locals.user.subscription_active };
};

export const actions: Actions = {
    createStripeCheckout: async ({ locals }) => {
        const end = perf("create_stripe_checkout");
        const metadata = await createMetadata(locals.user.id);
        const s = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/StripeUrlResponse").StripeUrlResponse__Output>>((r) =>
            usersService.CreateStripeCheckout({}, metadata, grpcSafe(r)),
        );
        if (s.error) return fail(500, { error: s.msg });
        end();
        throw redirect(303, s.data.url ?? "");
    },
    createStripePortal: async ({ locals }) => {
        const end = perf("create_stripe_portal");
        const metadata = await createMetadata(locals.user.id);
        const s = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/StripeUrlResponse").StripeUrlResponse__Output>>((r) =>
            usersService.CreateStripePortal({}, metadata, grpcSafe(r)),
        );
        if (s.error) return fail(500, { error: s.msg });
        end();
        throw redirect(303, s.data.url ?? "");
    },
};
