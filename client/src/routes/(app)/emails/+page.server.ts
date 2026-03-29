import { getFormValue } from "$lib/utils";
import { grpcSafe, safe } from "$lib/safe";
import { utilsService } from "$lib/server/grpc";
import { perf } from "$lib/server/logger";
import { createMetadata } from "$lib/server/metadata";
import { fail } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import type { Count__Output } from "$lib/proto/proto/Count";
import type { Email__Output } from "$lib/proto/proto/Email";

export const load: PageServerLoad = async ({ locals, url }) => {
    const end = perf("load_emails");
    const metadata = await createMetadata(locals.user.id);

    const s1 = new Promise<import("$lib/safe").Safe<Count__Output>>((r) => {
        utilsService.CountEmailsByTargetId({}, metadata, grpcSafe(r));
    });

    const limit = 10;
    const offset = (Number(url.searchParams.get("p") ?? 1) - 1) * limit;
    const emailsStream = utilsService.GetEmailsByTargetId({ offset, limit }, metadata);
    const p2 = new Promise<Email__Output[]>((res, rej) => {
        const emails: Email__Output[] = [];
        emailsStream.on("data", (data: Email__Output) => emails.push(data));
        emailsStream.on("error", (err: Error) => rej(err));
        emailsStream.on("end", () => res(emails));
    });
    const s2 = safe(p2);

    const [d1, d2] = await Promise.all([s1, s2]);
    if (d1.error || d2.error) {
        return { error: "Failed to load emails", emails: [], total: 0, pageSize: limit };
    }

    end();
    return {
        error: "",
        emails: d2.data.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime()),
        total: Number(d1.data.count),
        pageSize: limit,
    };
};

export const actions: Actions = {
    sendEmail: async ({ locals, request }) => {
        const end = perf("send_email");
        const form = await request.formData();
        const data = {
            email_to: getFormValue(form, "email_to"),
            email_from: getFormValue(form, "email_from"),
            email_from_name: getFormValue(form, "email_from_name"),
            email_subject: getFormValue(form, "email_subject"),
            email_body: getFormValue(form, "email_body"),
        };
        const res = await new Promise<import("$lib/safe").Safe<Email__Output>>((r) => {
            utilsService.SendEmail(data, await createMetadata(locals.user.id), grpcSafe(r));
        });

        if (res.error) {
            if (res.fields) return fail(400, { fields: res.fields });
            return fail(400, { error: res.msg });
        }
        end();
        return { profile: res.data };
    },
};
