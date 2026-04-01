<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        type?: "button" | "submit" | "reset";
        variant?: "primary" | "secondary" | "link" | "danger";
        form?: string;
        href?: string;
        loading?: boolean;
        disabled?: boolean;
        class?: string;
        icon?: Snippet;
        children?: Snippet;
        onclick?: (e: MouseEvent) => void;
    }

    let {
        type = "submit",
        variant = "primary",
        form,
        href = "",
        loading = false,
        disabled = false,
        class: className = "",
        icon,
        children,
        onclick,
    }: Props = $props();

    const base =
        "group inline-flex items-center justify-center gap-x-2 rounded-md px-3 py-2 text-sm font-semibold transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2";
    const variants: Record<string, string> = {
        primary: "bg-indigo-600 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline-indigo-600",
        secondary: "bg-white text-gray-900 shadow-sm hover:bg-gray-50 focus-visible:outline-gray-300",
        link: "text-indigo-600 hover:text-indigo-500 focus-visible:outline-indigo-600",
        danger: "bg-red-600 text-white shadow-sm hover:bg-red-500 focus-visible:outline-red-600",
    };
</script>

{#if !href}
    <button
        {onclick}
        {form}
        {type}
        disabled={loading || disabled}
        class="{base} {variants[variant]} {className}"
    >
        {#if loading}
            <svg class="h-5 w-5 animate-spin text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
        {:else if icon}
            {@render icon()}
        {/if}
        {@render children?.()}
    </button>
{:else}
    <a
        {href}
        class="group inline-flex items-center justify-center gap-x-1.5 rounded-md px-3 py-2 text-sm font-semibold no-underline transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 {variants[variant]} {className}"
    >
        {@render children?.()}
    </a>
{/if}
