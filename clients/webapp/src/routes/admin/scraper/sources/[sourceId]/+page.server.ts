import { grpcSafe } from "$lib/safe";
import { scraperService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import type { Source__Output } from "$lib/proto/proto/Source";

export const load: PageServerLoad = async ({ locals, params }) => {
    const metadata = await createMetadata(locals.user.id);
    const res = await new Promise<import("$lib/safe").Safe<Source__Output>>((r) => {
        scraperService.GetSource({ id: params.sourceId }, metadata, grpcSafe(r));
    });
    if (res.error) redirect(302, "/admin/scraper/sources");
    return { source: res.data };
};

export const actions: Actions = {
    update: async ({ locals, request, params }) => {
        const form = await request.formData();
        const name = form.get("name")?.toString().trim() ?? "";
        const source_url = form.get("source_url")?.toString().trim() ?? "";
        const source_type = form.get("source_type")?.toString().trim() ?? "";
        const integration_mode = form.get("integration_mode")?.toString() ?? "MANUAL";
        const auto_schedule = form.get("auto_schedule")?.toString().trim() ?? "";
        const field_rules = form.get("field_rules")?.toString().trim() || "[]";
        const active = form.get("active") === "true";

        // Validate field_rules JSON
        try { JSON.parse(field_rules); } catch {
            return fail(400, { error: "field_rules must be valid JSON" });
        }

        const metadata = await createMetadata(locals.user.id);
        const res = await new Promise<import("$lib/safe").Safe<Source__Output>>((r) => {
            scraperService.UpdateSource(
                { id: params.sourceId, name, source_url, source_type, integration_mode, auto_schedule, field_rules, active },
                metadata,
                grpcSafe(r),
            );
        });

        if (res.error) return fail(500, { error: res.msg });
        return { success: true };
    },

    delete: async ({ locals, params }) => {
        const metadata = await createMetadata(locals.user.id);
        await new Promise<void>((resolve) => {
            scraperService.DeleteSource({ id: params.sourceId }, metadata, () => resolve());
        });
        redirect(302, "/admin/scraper/sources");
    },
};
