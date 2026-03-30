import { ENV } from "$env/static/private";
import pino from "pino";

export const logger = pino({
    transport: {
        target: "pino-pretty",
        options: { colorize: true },
    },
    level: ENV === "production" ? "info" : "debug",
});

export function perf(name: string): () => void {
    const start = performance.now();
    return function end() {
        const duration = performance.now() - start;
        logger.info(`${name}: ${duration.toFixed(4)}ms`);
    };
}
