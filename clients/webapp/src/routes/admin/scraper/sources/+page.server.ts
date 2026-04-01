import { grpcSafe, safe } from "$lib/safe";
import { scraperService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import type { Source__Output } from "$lib/proto/proto/Source";

export const load: PageServerLoad = async ({ locals }) => {
    const metadata = await createMetadata(locals.user.id);

    const stream = scraperService.ListSources({}, metadata);
    const p = new Promise<Source__Output[]>((res, rej) => {
        const items: Source__Output[] = [];
        stream.on("data", (s: Source__Output) => items.push(s));
        stream.on("error", (e: Error) => rej(e));
        stream.on("end", () => res(items));
    });

    const d = await safe(p);
    return { sources: d.error ? [] : d.data };
};

export const actions: Actions = {
    create: async ({ locals, request }) => {
        const form = await request.formData();
        const name = form.get("name")?.toString().trim() ?? "";
        const source_url = form.get("source_url")?.toString().trim() ?? "";
        const source_type = form.get("source_type")?.toString().trim() ?? "";
        const integration_mode = form.get("integration_mode")?.toString() ?? "MANUAL";
        const auto_schedule = form.get("auto_schedule")?.toString().trim() ?? "";

        if (!name || !source_url || !source_type) {
            return fail(400, { error: "Name, URL and type are required" });
        }

        const metadata = await createMetadata(locals.user.id);
        const res = await new Promise<import("$lib/safe").Safe<Source__Output>>((r) => {
            scraperService.CreateSource(
                { name, source_url, source_type, integration_mode, auto_schedule, field_rules: "[]", active: true },
                metadata,
                grpcSafe(r),
            );
        });

        if (res.error) return fail(500, { error: res.msg });
        redirect(302, `/admin/scraper/sources/${res.data.id}`);
    },
};
