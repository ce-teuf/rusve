<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import { cn } from "$lib/utils";
    import type { HTMLInputAttributes } from "svelte/elements";

    interface Props extends HTMLInputAttributes {
        name: string;
        label: string;
        value: string | number;
        rows?: number;
        error?: string;
        helper?: string;
    }

    let {
        name,
        label,
        value = $bindable(),
        rows = 0,
        error = "",
        helper = "\x80",
        class: className = "",
        ...restProps
    }: Props = $props();
</script>

<div class={className}>
    <label for={name} class="block text-sm font-medium leading-6">{label}</label>
    <div class="mt-2">
        {#if rows === 0}
            <Input
                bind:value
                id={name}
                {name}
                aria-invalid={!!error}
                aria-describedby="{name}-description"
                class={cn(error && "outline outline-1 outline-red-600")}
                {...restProps}
            />
        {:else}
            <textarea
                bind:value
                id={name}
                {name}
                {rows}
                aria-invalid={!!error}
                aria-describedby="{name}-description"
                class="block w-full rounded-md border-0 bg-gray-800 px-3 py-1.5 shadow-inset placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6 {error
                    ? 'outline outline-1 outline-red-600'
                    : ''}"
            ></textarea>
        {/if}
    </div>
    <p id="{name}-description" class="mb-2 mt-0 text-sm leading-6 {error ? 'text-red-600' : 'text-gray-500'}">
        {error || helper}
    </p>
</div>
