<script lang="ts">
    import { cubicIn, cubicOut } from "svelte/easing";
    import { fade, scale } from "svelte/transition";
    import { checkElement, generateId } from "$lib/utils";
    import Button from "$lib/form/Button.svelte";
    import { browser } from "$app/environment";
    import type { Snippet } from "svelte";

    interface Props {
        open: boolean;
        title: string;
        description: string;
        alert?: boolean;
        children?: Snippet;
    }

    let {
        open = $bindable(),
        title,
        description,
        alert = true,
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
            if (event.key === "Escape") { open = false; }
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
            if (!node.contains(event.target)) open = false;
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

<div
    class="relative z-50"
    role={alert ? "alertdialog" : "dialog"}
    aria-labelledby="modal-title-{id}"
    aria-describedby="modal-description-{id}"
    aria-modal="true"
>
    <div
        in:fade={{ duration: 300, easing: cubicOut }}
        out:fade={{ duration: 200, easing: cubicIn }}
        class="fixed inset-0 bg-gray-900 bg-opacity-75 transition-opacity"
    />
    <div class="fixed inset-0 z-10 w-screen overflow-y-auto">
        <div class="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
            <div
                use:portal
                in:scale={{ duration: 300, easing: cubicOut, start: 0.95 }}
                out:scale={{ duration: 200, easing: cubicIn, start: 0.95 }}
                class="relative transform overflow-hidden rounded-lg bg-gray-800 px-4 pb-4 pt-5 text-left shadow-xl ring-1 ring-gray-700 transition-all sm:my-8 sm:w-full sm:max-w-lg sm:p-6"
            >
                <div class="sm:flex sm:items-start">
                    <div class="mx-auto flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
                        <svg class="h-6 w-6 text-red-600" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                        </svg>
                    </div>
                    <div class="mt-3 text-center sm:ml-4 sm:mt-0 sm:text-left">
                        <h3 class="text-base font-semibold leading-6 text-gray-100" id="modal-title-{id}">{title}</h3>
                        <div class="mt-2" id="modal-description-{id}">
                            <p class="text-sm text-gray-200">{description}</p>
                        </div>
                    </div>
                </div>
                <div class="mt-5 flex flex-col gap-2 sm:float-right sm:mt-4 sm:inline-flex sm:flex-row-reverse">
                    <Button onclick={() => (open = false)} type="button" variant="secondary">Cancel</Button>
                    {@render children?.()}
                </div>
            </div>
        </div>
    </div>
</div>
