import { logger } from "$lib/server/logger";
import { Status } from "@grpc/grpc-js/build/src/constants";
import type { ServiceError } from "@grpc/grpc-js";

export type Safe<T> =
    | { error: false; data: T }
    | { error: true; msg: string; fields?: { field: string; tag: string }[] };

export function safe<T>(promise: Promise<T>): Promise<Safe<T>>;
export function safe<T>(fn: () => T): Safe<T>;
export function safe<T>(
    promiseOrFunc: Promise<T> | (() => T),
): Promise<Safe<T>> | Safe<T> {
    if (promiseOrFunc instanceof Promise) {
        return safeAsync(promiseOrFunc);
    }
    return safeSync(promiseOrFunc);
}

async function safeAsync<T>(promise: Promise<T>): Promise<Safe<T>> {
    try {
        const data = await promise;
        return { data, error: false };
    } catch (e) {
        logger.error(e);
        if (e instanceof Error) return { error: true, msg: e.message };
        return { error: true, msg: "Something went wrong" };
    }
}

function safeSync<T>(func: () => T): Safe<T> {
    try {
        const data = func();
        return { data, error: false };
    } catch (e) {
        logger.error(e);
        if (e instanceof Error) return { error: true, msg: e.message };
        return { error: true, msg: "Something went wrong" };
    }
}

export function grpcSafe<T>(
    res: (value: Safe<T>) => void,
): (err: ServiceError | null, data: T | undefined) => void {
    return (err, data) => {
        if (err) {
            logger.error(err);
            if (err.code === Status.INVALID_ARGUMENT) {
                let fields: { field: string; tag: string }[] = [];
                try {
                    fields = JSON.parse(err.details);
                } catch {
                    return res({ error: true, msg: err?.message || "Something went wrong" });
                }
                return res({ error: true, msg: "Invalid argument", fields });
            }
            return res({ error: true, msg: err?.message || "Something went wrong" });
        }
        if (!data) {
            logger.error("No data returned");
            return res({ error: true, msg: "No data returned" });
        }
        res({ data, error: false });
    };
}
