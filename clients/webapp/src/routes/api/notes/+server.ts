import { json } from "@sveltejs/kit";
import { grpcSafe } from "$lib/safe";
import { notesService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import type { RequestHandler } from "./$types";
import type { NoteResponse__Output } from "$lib/proto/proto/NoteResponse";

export const GET: RequestHandler = async ({ request, url }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const offset = (Number(url.searchParams.get("p") ?? 1) - 1) * 10;
    const limit = 10;

    const countReq = new Promise<number>((res, rej) => {
        notesService.CountNotesByUserId({}, metadata, (err, data) => {
            if (err || !data) return rej(err);
            res(Number(data.count));
        });
    });

    const notesStream = notesService.GetNotesByUserId({ offset, limit }, metadata);
    const notesReq = new Promise<NoteResponse__Output[]>((res, rej) => {
        const notes: NoteResponse__Output[] = [];
        notesStream.on("data", (note: NoteResponse__Output) => notes.push(note));
        notesStream.on("error", (err: Error) => rej(err));
        notesStream.on("end", () => res(notes));
    });

    const [total, notes] = await Promise.all([countReq, notesReq]).catch((err) => {
        throw json({ error: err?.message ?? "Failed to load notes" }, { status: 500 });
    });

    return json({
        notes: notes.sort(
            (a, b) =>
                new Date(b.note?.created ?? 0).getTime() -
                new Date(a.note?.created ?? 0).getTime(),
        ),
        total,
        pageSize: limit,
    });
};

export const POST: RequestHandler = async ({ request }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const body = await request.json().catch(() => ({}));
    const data = {
        title: String(body.title ?? ""),
        content: String(body.content ?? ""),
    };

    const metadata = await createMetadata(auth.user.id);
    const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Note").Note__Output>>((r) => {
        notesService.CreateNote(data, metadata, grpcSafe(r));
    });

    if (req.error) {
        if (req.fields) return json({ error: "Invalid fields", fields: req.fields }, { status: 400 });
        return json({ error: req.msg }, { status: 500 });
    }

    return json({ note: req.data }, { status: 201 });
};
