import { json } from "@sveltejs/kit";
import { safe, grpcSafe } from "$lib/safe";
import { utilsService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ request, params }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const stream = utilsService.GetFileById({ id: params.id }, metadata);

    let file: import("$lib/proto/proto/File").File__Output | undefined;
    const chunks: Buffer[] = [];
    const p = new Promise<import("$lib/proto/proto/File").File__Output>((res, rej) => {
        stream.on("error", (err: Error) => rej(err));
        stream.on("data", (data: import("$lib/proto/proto/File").File__Output) => {
            chunks.push(data.file_buffer as unknown as Buffer);
            file = data;
        });
        stream.on("end", () => res(file!));
    });
    const s = await safe(p);
    if (s.error) return json({ error: s.msg }, { status: 500 });

    return json({
        fileName: s.data.file_name,
        fileType: s.data.file_type,
        fileBuffer: Array.from(Buffer.concat(chunks)),
    });
};

export const DELETE: RequestHandler = async ({ request, params }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const req = await new Promise<import("$lib/safe").Safe<import("$lib/proto/proto/Empty").Empty>>((r) => {
        utilsService.DeleteFileById({ id: params.id }, metadata, grpcSafe(r));
    });

    if (req.error) return json({ error: req.msg }, { status: 400 });
    return json({ success: true });
};
