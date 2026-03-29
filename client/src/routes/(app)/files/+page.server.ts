import { grpcSafe, safe } from "$lib/safe";
import { utilsService } from "$lib/server/grpc";
import { perf } from "$lib/server/logger";
import { createMetadata } from "$lib/server/metadata";
import { fail } from "@sveltejs/kit";
import { FileTarget } from "$lib/proto/proto/FileTarget";
import { getFormValue } from "$lib/utils";
import type { PageServerLoad, Actions } from "./$types";
import type { Count__Output } from "$lib/proto/proto/Count";
import type { File__Output } from "$lib/proto/proto/File";

export const load: PageServerLoad = async ({ locals, url }) => {
    const end = perf("load_files");
    const metadata = await createMetadata(locals.user.id);

    const s1 = new Promise<import("$lib/safe").Safe<Count__Output>>((r) => {
        utilsService.CountFilesByTargetId({}, metadata, grpcSafe(r));
    });

    const limit = 10;
    const offset = (Number(url.searchParams.get("p") ?? 1) - 1) * limit;
    const filesStream = utilsService.GetFilesByTargetId({ offset, limit }, metadata);
    const p2 = new Promise<File__Output[]>((res, rej) => {
        const files: File__Output[] = [];
        filesStream.on("data", (data: File__Output) => files.push(data));
        filesStream.on("error", (err: Error) => rej(err));
        filesStream.on("end", () => res(files));
    });
    const s2 = safe(p2);

    const [d1, d2] = await Promise.all([s1, s2]);
    if (d1.error || d2.error) {
        return { error: "Failed to load files", files: [], total: 0, pageSize: limit };
    }

    end();
    return {
        error: "",
        files: d2.data
            .sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())
            .map((f) => ({ ...f, file_buffer: [] })),
        total: Number(d1.data.count),
        pageSize: limit,
    };
};

export const actions: Actions = {
    uploadFile: async ({ locals, request }) => {
        const end = perf("upload_file");
        const form = await request.formData();

        const file = form.get("file");
        if (!file || !(file instanceof File)) return fail(400, { error: "No file" });
        if (file.size > 1024 * 1024 * 10) return fail(400, { error: "File too large" });

        const arrayBuffer = await safe(file.arrayBuffer());
        if (arrayBuffer.error) return fail(400, { error: arrayBuffer.msg });
        const buffer = Buffer.from(arrayBuffer.data);

        const metadata = await createMetadata(locals.user.id);
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
            if (res.error) return fail(500, { error: res.msg });
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
        if (s.error) return fail(500, { error: s.msg });

        end();
        return { file: { ...s.data, file_buffer: [] } };
    },
    downloadFile: async ({ locals, request }) => {
        const end = perf("download_file");
        const form = await request.formData();
        const id = getFormValue(form, "id");

        const metadata = await createMetadata(locals.user.id);
        const stream = utilsService.GetFileById({ id }, metadata);

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
        if (s.error) return fail(500, { error: s.msg });

        end();
        return {
            fileName: s.data.file_name,
            fileType: s.data.file_type,
            fileBuffer: Array.from(Buffer.concat(chunks)),
        };
    },
};
