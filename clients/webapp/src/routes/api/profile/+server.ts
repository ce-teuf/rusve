import { json } from "@sveltejs/kit";
import { grpcSafe } from "$lib/safe";
import { upsendApi } from "$lib/server/api";
import { usersService } from "$lib/server/grpc";
import { logger } from "$lib/server/logger";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import { safe } from "$lib/safe";
import type { RequestHandler } from "./$types";
import type { UpsendFile, UpsendImage } from "$lib/types";

export const GET: RequestHandler = async ({ request }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const profile = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Profile").Profile__Output>>((r) => {
        usersService.GetProfileByUserId({}, metadata, grpcSafe(r));
    });

    if (profile.error) return json({ error: profile.msg }, { status: 500 });

    let resume: UpsendFile | undefined;
    if (profile.data.resume_id) {
        const r = await upsendApi<UpsendFile>({ url: `/files/${profile.data.resume_id}`, method: "GET" });
        if (!r.error) resume = r.data;
    }

    return json({ profile: profile.data, resume });
};

export const POST: RequestHandler = async ({ request }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const form = await request.formData().catch(() => null);
    if (!form) return json({ error: "Expected multipart/form-data" }, { status: 400 });

    let resume_id = String(form.get("resume_id") ?? "");
    const resume = form.get("resume");
    if (!(resume instanceof File)) return json({ error: "Resume must be a PDF" }, { status: 400 });
    if (resume.size > 0) {
        if (resume.size > 5 * 1024 * 1024) return json({ error: "Resume must be less than 5MB" }, { status: 400 });
        if (!resume.name.endsWith(".pdf")) return json({ error: "Resume must be a PDF" }, { status: 400 });

        if (resume_id) {
            const resDel = await upsendApi({ url: `/files/${resume_id}`, method: "DELETE" });
            if (resDel.error) return json({ error: resDel.msg }, { status: 400 });
        }

        const file = await upsendApi<UpsendFile>({ url: "/files", method: "POST", file: resume });
        if (file.error) return json({ error: file.msg }, { status: 400 });
        resume_id = file.data.id;
    }

    let cover_id = String(form.get("cover_id") ?? "");
    let cover_url = String(form.get("cover_url") ?? "");
    const cover = form.get("cover");
    if (!(cover instanceof File)) return json({ error: "Cover must be an image" }, { status: 400 });
    if (cover.size > 0) {
        if (cover.size > 5 * 1024 * 1024) return json({ error: "Cover must be less than 5MB" }, { status: 400 });
        const extensions = [".png", ".jpg", ".jpeg", ".gif", ".svg"];
        if (!extensions.some((ext) => cover.name.endsWith(ext))) {
            return json({ error: "Cover must be an image" }, { status: 400 });
        }

        if (cover_id) {
            const resDel = await upsendApi({ url: `/images/${cover_id}`, method: "DELETE" });
            if (resDel.error) return json({ error: resDel.msg }, { status: 400 });
        }

        const file = await upsendApi<UpsendImage>({ url: "/images", method: "POST", file: cover });
        if (file.error) return json({ error: file.msg }, { status: 400 });
        cover_id = file.data.id;
        cover_url = file.data.url;
    }

    const data = {
        id: String(form.get("id") ?? ""),
        name: String(form.get("name") ?? ""),
        about: String(form.get("about") ?? ""),
        resume_id,
        cover_id,
        cover_url,
    };

    const metadata = await createMetadata(auth.user.id);
    const res = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Profile").Profile__Output>>((r) => {
        usersService.CreateProfile(data, metadata, grpcSafe(r));
    });

    if (res.error) {
        if (res.fields) return json({ error: "Invalid fields", fields: res.fields }, { status: 400 });
        return json({ error: res.msg }, { status: 400 });
    }

    safe(
        upsendApi({
            url: "/emails",
            method: "POST",
            email: {
                email_to: auth.user.email,
                email_name: res.data.name ?? "",
                email_subject: "You've updated your profile",
                email_html: `<p>Hi ${res.data.name},</p><p>You've updated your profile.</p>`,
            },
        }),
    ).catch(() => logger.error(`Failed to send email to user: ${auth.user.email}`));

    return json({ profile: res.data });
};
