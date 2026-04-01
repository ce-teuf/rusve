import { json } from "@sveltejs/kit";
import { grpcSafe } from "$lib/safe";
import { notesService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ request, params }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Note").Note__Output>>((r) => {
        notesService.GetNoteById({ id: params.id }, metadata, grpcSafe(r));
    });

    if (req.error) return json({ error: req.msg }, { status: 404 });
    return json({ note: req.data });
};

export const PUT: RequestHandler = async ({ request, params }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const body = await request.json().catch(() => ({}));
    const data = {
        id: params.id,
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

    return json({ note: req.data });
};

export const DELETE: RequestHandler = async ({ request, params }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Empty").Empty>>((r) => {
        notesService.DeleteNoteById({ id: params.id }, metadata, grpcSafe(r));
    });

    if (req.error) return json({ error: req.msg }, { status: 400 });
    return json({ success: true });
};
