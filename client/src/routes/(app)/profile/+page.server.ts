import { getFormValue } from "$lib/utils";
import { safe, grpcSafe } from "$lib/safe";
import { upsendApi } from "$lib/server/api";
import { usersService } from "$lib/server/grpc";
import { logger, perf } from "$lib/server/logger";
import { createMetadata } from "$lib/server/metadata";
import { error, fail } from "@sveltejs/kit";
import type { PageServerLoad, Actions } from "./$types";
import type { UpsendFile, UpsendImage } from "$lib/types";

export const load: PageServerLoad = async ({ locals }) => {
    const end = perf("load_profile");
    const profile = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Profile").Profile__Output>>((r) => {
        usersService.GetProfileByUserId({}, await createMetadata(locals.user.id), grpcSafe(r));
    });
    if (profile.error) throw error(500, profile.msg);

    let resumePromise: Promise<import("$lib/safe").Safe<UpsendFile | undefined>> = Promise.resolve({ data: undefined, error: false });
    if (profile.data.resume_id) {
        resumePromise = upsendApi<UpsendFile>({ url: `/files/${profile.data.resume_id}`, method: "GET" });
    }

    end();
    return {
        profile: profile.data,
        stream: { resume: resumePromise },
    };
};

export const actions: Actions = {
    createProfile: async ({ locals, request }) => {
        const end = perf("create_profile");
        const form = await request.formData();

        let resume_id = getFormValue(form, "resume_id");
        const resume = form.get("resume");
        if (!(resume instanceof File)) return fail(400, { error: "Resume must be a PDF" });
        if (resume.size > 0) {
            if (resume.size > 5 * 1024 * 1024) return fail(400, { error: "Resume must be less than 5MB" });
            if (!resume.name.endsWith(".pdf")) return fail(400, { error: "Resume must be a PDF" });

            if (resume_id) {
                const resDel = await upsendApi({ url: `/files/${resume_id}`, method: "DELETE" });
                if (resDel.error) return fail(400, { error: resDel.msg });
            }

            const file = await upsendApi<UpsendFile>({ url: "/files", method: "POST", file: resume });
            if (file.error) return fail(400, { error: file.msg });
            resume_id = file.data.id;
        }

        let cover_id = getFormValue(form, "cover_id");
        let cover_url = getFormValue(form, "cover_url");
        const cover = form.get("cover");
        if (!(cover instanceof File)) return fail(400, { error: "Cover must be an image" });
        if (cover.size > 0) {
            if (cover.size > 5 * 1024 * 1024) return fail(400, { error: "Cover must be less than 5MB" });
            const extensions = [".png", ".jpg", ".jpeg", ".gif", ".svg"];
            if (!extensions.some((ext) => cover.name.endsWith(ext))) return fail(400, { error: "Cover must be an image" });

            if (cover_id) {
                const resDel = await upsendApi({ url: `/images/${cover_id}`, method: "DELETE" });
                if (resDel.error) return fail(400, { error: resDel.msg });
            }

            const file = await upsendApi<UpsendImage>({ url: "/images", method: "POST", file: cover });
            if (file.error) return fail(400, { error: file.msg });
            cover_id = file.data.id;
            cover_url = file.data.url;
        }

        const data = {
            id: getFormValue(form, "id"),
            name: getFormValue(form, "name"),
            about: getFormValue(form, "about"),
            resume_id,
            cover_id,
            cover_url,
        };

        const res = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Profile").Profile__Output>>((r) => {
            usersService.CreateProfile(data, await createMetadata(locals.user.id), grpcSafe(r));
        });

        if (res.error) {
            if (res.fields) return fail(400, { fields: res.fields });
            return fail(400, { error: res.msg });
        }

        safe(
            upsendApi({
                url: "/emails",
                method: "POST",
                email: {
                    email_to: locals.user.email,
                    email_name: res.data.name ?? "",
                    email_subject: "You've updated your profile",
                    email_html: `<p>Hi ${res.data.name},</p><p>You've updated your profile.</p>`,
                },
            }),
        ).catch(() => logger.error("Failed to send email to user", { email: locals.user.email }));

        end();
        return { profile: res.data };
    },
};
