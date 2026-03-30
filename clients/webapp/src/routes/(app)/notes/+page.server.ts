import { getFormValue } from "$lib/utils";
import { grpcSafe, safe } from "$lib/safe";
import { notesService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { fail } from "@sveltejs/kit";
import { perf } from "$lib/server/logger";
import type { PageServerLoad, Actions } from "./$types";
import type { Count__Output } from "$lib/proto/proto/Count";
import type { NoteResponse__Output } from "$lib/proto/proto/NoteResponse";

export const load: PageServerLoad = async ({ locals, url }) => {
    const end = perf("load_notes");
    const metadata = await createMetadata(locals.user.id);

    const s1 = new Promise<import("$lib/safe").Safe<Count__Output>>((r) => {
        notesService.CountNotesByUserId({}, metadata, grpcSafe(r));
    });

    const offset = Number(url.searchParams.get("p") ?? 1) - 1;
    const limit = 10;

    const notesStream = notesService.GetNotesByUserId({ offset: offset * limit, limit }, metadata);
    const p2 = new Promise<NoteResponse__Output[]>((res, rej) => {
        const notes: NoteResponse__Output[] = [];
        notesStream.on("data", (note: NoteResponse__Output) => notes.push(note));
        notesStream.on("error", (err: Error) => rej(err));
        notesStream.on("end", () => res(notes));
    });
    const s2 = safe(p2);

    const [d1, d2] = await Promise.all([s1, s2]);

    if (d1.error) return { error: d1.msg, notes: [], total: 0, pageSize: limit };
    if (d2.error) return { error: d2.msg, notes: [], total: 0, pageSize: limit };

    end();
    return {
        notes: d2.data.sort(
            (a, b) =>
                new Date(b.note?.created ?? 0).getTime() -
                new Date(a.note?.created ?? 0).getTime(),
        ),
        total: Number(d1.data.count),
        pageSize: limit,
    };
};

export const actions: Actions = {
    insert: async ({ locals, request }) => {
        const end = perf("insert_note");
        const form = await request.formData();
        const data = {
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
};
