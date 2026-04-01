import { grpcSafe, safe } from "$lib/safe";
import { scraperService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import type { Job__Output } from "$lib/proto/proto/Job";
import type { Item__Output } from "$lib/proto/proto/Item";

export const load: PageServerLoad = async ({ locals, params, url }) => {
    const metadata = await createMetadata(locals.user.id);
    const statusFilter = url.searchParams.get("status") ?? "";

    const p1 = new Promise<import("$lib/safe").Safe<Job__Output>>((r) => {
        scraperService.GetJobById({ id: params.jobId }, metadata, grpcSafe(r));
    });

    const itemsStream = scraperService.ListItems(
        { job_id: params.jobId, status: statusFilter, offset: 0, limit: 100 },
        metadata,
    );
    const p2 = new Promise<Item__Output[]>((res, rej) => {
        const items: Item__Output[] = [];
        itemsStream.on("data", (i: Item__Output) => items.push(i));
        itemsStream.on("error", (e: Error) => rej(e));
        itemsStream.on("end", () => res(items));
    });

    const [d1, d2] = await Promise.all([p1, safe(p2)]);

    if (d1.error) redirect(302, "/admin/scraper");

    return {
        job: d1.data,
        items: d2.error ? [] : d2.data,
        statusFilter,
    };
};

export const actions: Actions = {
    approve: async ({ locals, request }) => {
        const form = await request.formData();
        const id = form.get("id")?.toString() ?? "";
        const metadata = await createMetadata(locals.user.id);
        const res = await new Promise<import("$lib/safe").Safe<Item__Output>>((r) => {
            scraperService.ApproveItem({ id }, metadata, grpcSafe(r));
        });
        if (res.error) return fail(500, { error: res.msg });
        return { success: true };
    },

    reject: async ({ locals, request }) => {
        const form = await request.formData();
        const id = form.get("id")?.toString() ?? "";
        const metadata = await createMetadata(locals.user.id);
        const res = await new Promise<import("$lib/safe").Safe<Item__Output>>((r) => {
            scraperService.RejectItem({ id }, metadata, grpcSafe(r));
        });
        if (res.error) return fail(500, { error: res.msg });
        return { success: true };
    },

    approveAllValid: async ({ locals, params }) => {
        const metadata = await createMetadata(locals.user.id);
        const res = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Count").Count__Output>>((r) => {
            scraperService.ApproveAllValid({ id: params.jobId }, metadata, grpcSafe(r));
        });
        if (res.error) return fail(500, { error: res.msg });
        return { approvedCount: Number(res.data.count) };
    },

    pushApproved: async ({ locals, params }) => {
        const metadata = await createMetadata(locals.user.id);
        const res = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Count").Count__Output>>((r) => {
            scraperService.PushApproved({ id: params.jobId }, metadata, grpcSafe(r));
        });
        if (res.error) return fail(500, { error: res.msg });
        return { pushedCount: Number(res.data.count) };
    },
};
