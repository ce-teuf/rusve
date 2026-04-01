import adapterNode from "@sveltejs/adapter-node";
import adapterStatic from "@sveltejs/adapter-static";
import type { Config } from "@sveltejs/kit";

const isMobile = process.env.BUILD_TARGET === "mobile";

const config: Config = {
    kit: {
        adapter: isMobile
            ? adapterStatic({
                  fallback: "index.html",
                  strict: false,
              })
            : adapterNode(),

        // When building for mobile (SPA), nothing is prerendered —
        // all data loading happens client-side via +page.ts → /api/* REST calls.
        prerender: isMobile
            ? {
                  entries: [],
                  handleMissingId: "warn",
                  handleHttpError: "warn",
              }
            : {},
    },
};

export default config;
