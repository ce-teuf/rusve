import { UPSEND_KEY } from "$env/static/private";
import { logger } from "./logger";
import type { Safe } from "$lib/safe";
import type { UpsendEmail } from "$lib/types";

export async function upsendApi<T>({
    method = "GET",
    url,
    file,
    email,
}: {
    method?: "GET" | "POST" | "DELETE";
    url: string;
    file?: File;
    email?: UpsendEmail;
}): Promise<Safe<T>> {
    try {
        const headers = new Headers();
        headers.append("Authorization", `Bearer ${UPSEND_KEY}`);

        let body: FormData | string | null = null;
        if (file) {
            body = new FormData();
            body.append("file", file);
        } else if (email) {
            body = JSON.stringify(email);
            headers.append("Content-Type", "application/json");
        }

        const response = await fetch("https://api.upsend.app" + url, {
            method,
            headers,
            body,
        });
        if (!response.ok) throw new Error(await response.text());
        if (response.status === 204) return { error: false, data: "" as T };

        const data = await response.json();
        return { error: false, data };
    } catch (error) {
        logger.error(error);
        return { error: true, msg: "Error using Upsend API" };
    }
}
