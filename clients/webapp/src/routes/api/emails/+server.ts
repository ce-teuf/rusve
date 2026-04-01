import { json } from "@sveltejs/kit";
import { grpcSafe } from "$lib/safe";
import { utilsService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import type { RequestHandler } from "./$types";
import type { Email__Output } from "$lib/proto/proto/Email";

export const GET: RequestHandler = async ({ request, url }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const limit = 10;
    const offset = (Number(url.searchParams.get("p") ?? 1) - 1) * limit;

    const countReq = new Promise<number>((res, rej) => {
        utilsService.CountEmailsByTargetId({}, metadata, (err, data) => {
            if (err || !data) return rej(err);
            res(Number(data.count));
        });
    });

    const emailsStream = utilsService.GetEmailsByTargetId({ offset, limit }, metadata);
    const emailsReq = new Promise<Email__Output[]>((res, rej) => {
        const emails: Email__Output[] = [];
        emailsStream.on("data", (data: Email__Output) => emails.push(data));
        emailsStream.on("error", (err: Error) => rej(err));
        emailsStream.on("end", () => res(emails));
    });

    const [total, emails] = await Promise.all([countReq, emailsReq]).catch((err) => {
        throw json({ error: err?.message ?? "Failed to load emails" }, { status: 500 });
    });

    return json({
        emails: emails.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime()),
        total,
        pageSize: limit,
    });
};

export const POST: RequestHandler = async ({ request }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const body = await request.json().catch(() => ({}));
    const data = {
        email_to: String(body.email_to ?? ""),
        email_from: String(body.email_from ?? ""),
        email_from_name: String(body.email_from_name ?? ""),
        email_subject: String(body.email_subject ?? ""),
        email_body: String(body.email_body ?? ""),
    };

    const metadata = await createMetadata(auth.user.id);
    const res = await new Promise<import("$lib/safe").Safe<Email__Output>>((r) => {
        utilsService.SendEmail(data, metadata, grpcSafe(r));
    });

    if (res.error) {
        if (res.fields) return json({ error: "Invalid fields", fields: res.fields }, { status: 400 });
        return json({ error: res.msg }, { status: 400 });
    }

    return json({ email: res.data }, { status: 201 });
};
