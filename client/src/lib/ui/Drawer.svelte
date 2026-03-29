<script lang="ts">
    import { browser } from "$app/environment";
    import { checkElement, generateId } from "$lib/utils";
    import { fade, fly } from "svelte/transition";
    import type { Snippet } from "svelte";

    interface Props {
        open: boolean;
        title?: string;
        position?: "right" | "left";
        close: () => void;
        children?: Snippet;
    }

    let {
        open,
        title = "",
        position = "right",
        close,
        children,
    }: Props = $props();

    let previous: HTMLElement | undefined = $state(undefined);

    $effect(() => {
        if (!open) previous?.focus({ preventScroll: true });
    });

    $effect(() => {
        if (!browser) return;
        if (open) {
            document.body.classList.add("no-scroll");
        } else {
            document.body.classList.remove("no-scroll");
        }
    });

    function portal(node: HTMLElement): { destroy(): void } {
        previous = checkElement(document.activeElement);

        const focusable = node.querySelectorAll<HTMLElement>(
            'a[href], button, textarea, input[type="text"], input[type="radio"], input[type="checkbox"], select',
        );
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        first?.focus({ preventScroll: true });

        function handleKeydown(event: KeyboardEvent): void {
            if (event.key === "Escape") { close(); }
            if (event.key === "Tab") {
                if (event.shiftKey) {
                    if (document.activeElement === first) { event.preventDefault(); last?.focus({ preventScroll: true }); }
                } else {
                    if (document.activeElement === last) { event.preventDefault(); first?.focus({ preventScroll: true }); }
                }
            }
        }

        function handleClickOutside(event: MouseEvent): void {
            if (!(event.target instanceof Node)) return;
            if (previous?.contains(event.target)) return;
            if (!node.contains(event.target)) close();
        }

        document.addEventListener("keydown", handleKeydown);
        document.addEventListener("mousedown", handleClickOutside);

        return {
            destroy() {
                document.removeEventListener("keydown", handleKeydown);
                document.removeEventListener("mousedown", handleClickOutside);
            },
        };
    }

    const id = generateId();
</script>

<div class="relative z-50" aria-labelledby="drawer-title-{id}" role="dialog" aria-modal="true">
    <div transition:fade class="fixed inset-0 bg-gray-900 bg-opacity-75 transition-opacity" />
    <div class="fixed inset-0 overflow-hidden">
        <div class="absolute inset-0 overflow-hidden">
            <div class="pointer-events-none fixed inset-y-0 flex max-w-full {position === 'right' ? 'right-0 pl-10' : 'left-0 pr-10'}">
                <div
                    use:portal
                    in:fly={{ x: position === "right" ? "100%" : "-100%", duration: 400, opacity: 100 }}
                    out:fly={{ x: position === "right" ? "100%" : "-100%", duration: 400, opacity: 100 }}
                    class="pointer-events-auto relative w-screen max-w-xl"
                >
                    <div
                        class="absolute top-0 flex pt-4 {position === 'left' ? 'right-0 -mr-8 pl-2 sm:-mr-10 sm:pl-4' : 'left-0 -ml-8 pr-2 sm:-ml-10 sm:pr-4'}"
                        transition:fade
                    >
                        <button
                            type="button"
                            class="relative rounded-md text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-white"
                            onclick={close}
                        >
                            <span class="absolute -inset-2.5"></span>
                            <span class="sr-only">Close panel</span>
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                    {#if title}
                        <div class="flex h-full flex-col bg-gray-900 py-6 shadow-xl ring-1 ring-white/10">
                            <div class="px-4 sm:px-6">
                                <h2 class="text-base font-semibold leading-6 text-gray-50" id="slide-over-title">{title}</h2>
                            </div>
                            <div class="relative mt-6 flex-1 px-4 sm:px-6">
                                {@render children?.()}
                            </div>
                        </div>
                    {:else}
                        <div class="flex h-full flex-col bg-gray-900 shadow-xl ring-1 ring-white/10">
                            {@render children?.()}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
</div>
