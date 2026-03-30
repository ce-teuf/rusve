import { error, fail } from "@sveltejs/kit";
import { getFormValue } from "$lib/utils";
import { createMetadata } from "$lib/server/metadata";
import { notesService } from "$lib/server/grpc";
import { grpcSafe } from "$lib/safe";
import { perf } from "$lib/server/logger";
import type { PageServerLoad, Actions } from "./$types";

export const load: PageServerLoad = async ({ locals, params }) => {
    const end = perf("load_note");
    const id = params.noteId;
    if (!id) throw error(409, "Missing note id");

    const metadata = await createMetadata(locals.user.id);
    const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Note").Note__Output>>((r) => {
        notesService.GetNoteById({ id }, metadata, grpcSafe(r));
    });

    if (req.error) throw error(404, req.msg);

    end();
    return { note: req.data };
};

export const actions: Actions = {
    update: async ({ locals, request }) => {
        const end = perf("update_note");
        const form = await request.formData();
        const data = {
            id: getFormValue(form, "id"),
            title: getFormValue(form, "title"),
            content: getFormValue(form, "content"),
        };
        const metadata = await createMetadata(locals.user.id);
        const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Note").Note__Output>>((r) => {
            notesService.CreateNote(data, metadata, grpcSafe(r));
        });

        if (req.error) {
            if (req.fields) return fail(400, { fields: req.fields });
            return fail(500, { error: req.msg });
        }
        end();
        return { note: req.data };
    },
    delete: async ({ locals, request }) => {
        const end = perf("delete_note");
        const form = await request.formData();
        const data = { id: getFormValue(form, "id") };
        const metadata = await createMetadata(locals.user.id);
        const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Empty").Empty>>((r) => {
            notesService.DeleteNoteById(data, metadata, grpcSafe(r));
        });

        if (req.error) return fail(400, { error: req.msg });
        end();
        return { success: true };
    },
};
