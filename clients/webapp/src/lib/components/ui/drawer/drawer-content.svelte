<script lang="ts">
    import { Drawer as DrawerPrimitive } from "vaul-svelte";
    import { cn } from "$lib/utils.js";
    import type { Snippet } from "svelte";

    let {
        ref = $bindable(null),
        class: className,
        children,
        ...restProps
    }: DrawerPrimitive.ContentProps & { children?: Snippet } = $props();
</script>

<DrawerPrimitive.Portal>
    <DrawerPrimitive.Overlay class="fixed inset-0 z-50 bg-black/80" />
    <DrawerPrimitive.Content
        bind:ref
        class={cn(
            "fixed inset-x-0 bottom-0 z-50 mt-24 flex h-auto flex-col rounded-t-[10px] border border-gray-700 bg-gray-900",
            className
        )}
        {...restProps}
    >
        <DrawerPrimitive.Handle class="mx-auto mt-4 h-2 w-[100px] rounded-full bg-gray-700" />
        {@render children?.()}
    </DrawerPrimitive.Content>
</DrawerPrimitive.Portal>
