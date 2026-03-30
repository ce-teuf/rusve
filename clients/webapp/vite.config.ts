import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
    plugins: [sveltekit(), tailwindcss()],
    test: {
        include: ["src/**/*.{test,spec}.{js,ts}"],
    },
    server: {
        port: 3000,
        host: "0.0.0.0",
    },
    optimizeDeps: {
        exclude: ["jsonwebtoken"],
    },
    ssr: {
        external: ["jsonwebtoken"],
    },
});
