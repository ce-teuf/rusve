import { json } from "@sveltejs/kit";
import { safe, grpcSafe } from "$lib/safe";
import { utilsService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import { apiAuth } from "$lib/server/api-auth";
import { FileTarget } from "$lib/proto/proto/FileTarget";
import type { RequestHandler } from "./$types";
import type { File__Output } from "$lib/proto/proto/File";

export const GET: RequestHandler = async ({ request, url }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const metadata = await createMetadata(auth.user.id);
    const limit = 10;
    const offset = (Number(url.searchParams.get("p") ?? 1) - 1) * limit;

    const countReq = new Promise<number>((res, rej) => {
        utilsService.CountFilesByTargetId({}, metadata, (err, data) => {
            if (err || !data) return rej(err);
            res(Number(data.count));
        });
    });

    const filesStream = utilsService.GetFilesByTargetId({ offset, limit }, metadata);
    const filesReq = new Promise<File__Output[]>((res, rej) => {
        const files: File__Output[] = [];
        filesStream.on("data", (data: File__Output) => files.push(data));
        filesStream.on("error", (err: Error) => rej(err));
        filesStream.on("end", () => res(files));
    });

    const [total, files] = await Promise.all([countReq, filesReq]).catch((err) => {
        throw json({ error: err?.message ?? "Failed to load files" }, { status: 500 });
    });

    return json({
        files: files
            .sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())
            .map((f) => ({ ...f, file_buffer: [] })),
        total,
        pageSize: limit,
    });
};

export const POST: RequestHandler = async ({ request }) => {
    const auth = await apiAuth(request);
    if (auth instanceof Response) return auth;

    const form = await request.formData().catch(() => null);
    if (!form) return json({ error: "Expected multipart/form-data" }, { status: 400 });

    const file = form.get("file");
    if (!file || !(file instanceof File)) return json({ error: "No file" }, { status: 400 });
    if (file.size > 1024 * 1024 * 10) return json({ error: "File too large (max 10MB)" }, { status: 400 });

    const arrayBuffer = await safe(file.arrayBuffer());
    if (arrayBuffer.error) return json({ error: arrayBuffer.msg }, { status: 400 });
    const buffer = Buffer.from(arrayBuffer.data);

    const metadata = await createMetadata(auth.user.id);
    const stream = utilsService.UploadFile(metadata);

    const chunkSize = 1024 * 64;
    let offset = 0;
    while (offset < buffer.length) {
        const chunk = buffer.subarray(offset, offset + chunkSize);
        const message = {
            file_name: file.name,
            file_size: String(file.size),
            file_type: file.type,
            file_target: FileTarget.DOCUMENT,
            file_buffer: chunk,
        };
        const res = safe(() => stream.write(message));
        if (res.error) return json({ error: res.msg }, { status: 500 });
        offset += chunkSize;
    }
    stream.end();

    let newFile: import("$lib/proto/proto/File").File__Output | undefined;
    const p = new Promise<import("$lib/proto/proto/File").File__Output>((res, rej) => {
        stream.on("error", (err: Error) => rej(err));
        stream.on("data", (data: import("$lib/proto/proto/File").File__Output) => { newFile = data; });
        stream.on("end", () => res(newFile!));
    });
    const s = await safe(p);
    if (s.error) return json({ error: s.msg }, { status: 500 });

    return json({ file: { ...s.data, file_buffer: [] } }, { status: 201 });
};
