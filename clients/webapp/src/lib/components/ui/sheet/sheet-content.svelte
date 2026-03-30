<script lang="ts">
    import { Dialog as SheetPrimitive } from "bits-ui";
    import { cn } from "$lib/utils.js";
    import type { Snippet } from "svelte";

    type Side = "left" | "right" | "top" | "bottom";

    interface Props extends SheetPrimitive.ContentProps {
        side?: Side;
        children?: Snippet;
    }

    let { ref = $bindable(null), class: className, side = "right", children, ...restProps }: Props = $props();

    const sides: Record<Side, string> = {
        right: "inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm",
        left:  "inset-y-0 left-0  h-full w-3/4 border-r sm:max-w-sm",
        top:   "inset-x-0 top-0   w-full border-b",
        bottom:"inset-x-0 bottom-0 w-full border-t",
    };
</script>

<SheetPrimitive.Portal>
    <SheetPrimitive.Overlay
        class="fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
    />
    <SheetPrimitive.Content
        bind:ref
        class={cn(
            "fixed z-50 flex flex-col bg-gray-900 shadow-xl transition ease-in-out",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:duration-300 data-[state=open]:duration-500",
            side === "right"  && "data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right",
            side === "left"   && "data-[state=closed]:slide-out-to-left  data-[state=open]:slide-in-from-left",
            side === "top"    && "data-[state=closed]:slide-out-to-top    data-[state=open]:slide-in-from-top",
            side === "bottom" && "data-[state=closed]:slide-out-to-bottom data-[state=open]:slide-in-from-bottom",
            sides[side],
            className
        )}
        {...restProps}
    >
        {@render children?.()}
        <SheetPrimitive.Close
            class="absolute right-4 top-4 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-indigo-600 disabled:pointer-events-none"
        >
            <svg class="h-5 w-5 text-gray-300" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
            <span class="sr-only">Close</span>
        </SheetPrimitive.Close>
    </SheetPrimitive.Content>
</SheetPrimitive.Portal>
