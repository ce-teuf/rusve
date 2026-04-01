import { error, fail } from "@sveltejs/kit";
import { getFormValue } from "$lib/utils";
import { createMetadata } from "$lib/server/metadata";
import { notesService, usersService } from "$lib/server/grpc";
import { grpcSafe } from "$lib/safe";
import { perf } from "$lib/server/logger";
import type { PageServerLoad, Actions } from "./$types";
import type { Note__Output } from "$lib/proto/proto/Note";
import type { Profile__Output } from "$lib/proto/proto/Profile";

export const load: PageServerLoad = async ({ locals, params }) => {
    const end = perf("load_note");
    const id = params.noteId;
    if (!id) throw error(409, "Missing note id");

    const metadata = await createMetadata(locals.user.id);
    const noteReq = await new Promise<import("$lib/safe").Safe<Note__Output>>((r) => {
        notesService.GetNoteById({ id }, metadata, grpcSafe(r));
    });

    if (noteReq.error) throw error(404, noteReq.msg);
    if (!noteReq.data) throw error(404, "Note not found");

    const profileReq = await new Promise<import("$lib/safe").Safe<Profile__Output>>((r) => {
        usersService.GetProfileByUserId({}, metadata, grpcSafe(r));
    });

    end();
    return {
        note: noteReq.data,
        profile: profileReq.data,
        email: locals.user.email,
        avatar: locals.user.avatar,
    };
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
