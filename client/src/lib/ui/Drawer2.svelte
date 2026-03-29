<script lang="ts">
    import * as Sheet from "$lib/components/ui/sheet/index.js";
    import type { Snippet } from "svelte";

    interface Props {
        open: boolean;
        title?: string;
        position?: "right" | "left";
        close: () => void;
        children?: Snippet;
    }

    let { open = $bindable(), title = "", position = "right", close, children }: Props = $props();
</script>

<Sheet.Root bind:open onOpenChange={(v) => { if (!v) close(); }}>
    <Sheet.Content side={position}>
        {#if title}
            <Sheet.Header>
                <Sheet.Title>{title}</Sheet.Title>
            </Sheet.Header>
        {/if}
        <div class="relative flex-1 px-4 sm:px-6">
            {@render children?.()}
        </div>
    </Sheet.Content>
</Sheet.Root>
