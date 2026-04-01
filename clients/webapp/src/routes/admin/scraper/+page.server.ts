import { grpcSafe, safe } from "$lib/safe";
import { scraperService } from "$lib/server/grpc";
import { createMetadata } from "$lib/server/metadata";
import type { PageServerLoad } from "./$types";
import type { JobResponse__Output } from "$lib/proto/proto/JobResponse";
import type { Source__Output } from "$lib/proto/proto/Source";

export const load: PageServerLoad = async ({ locals }) => {
    const metadata = await createMetadata(locals.user.id);

    const jobsStream = scraperService.ListJobs({ offset: 0, limit: 50 }, metadata);
    const p1 = new Promise<JobResponse__Output[]>((res, rej) => {
        const items: JobResponse__Output[] = [];
        jobsStream.on("data", (j: JobResponse__Output) => items.push(j));
        jobsStream.on("error", (e: Error) => rej(e));
        jobsStream.on("end", () => res(items));
    });

    const sourcesStream = scraperService.ListSources({}, metadata);
    const p2 = new Promise<Source__Output[]>((res, rej) => {
        const items: Source__Output[] = [];
        sourcesStream.on("data", (s: Source__Output) => items.push(s));
        sourcesStream.on("error", (e: Error) => rej(e));
        sourcesStream.on("end", () => res(items));
    });

    const [d1, d2] = await Promise.all([safe(p1), safe(p2)]);

    return {
        jobs: d1.error ? [] : d1.data,
        sources: d2.error ? [] : d2.data,
    };
};
