import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ data }) => {
    // subscriptionActive comes from +page.server.ts on web, or locals on SSR.
    // On mobile, subscription status is not known at load time —
    // the user can tap "Subscribe" which calls POST /api/subscription directly.
    if (data && "subscriptionActive" in data) return data;
    return { subscriptionActive: false };
};
