<script lang="ts">
    interface Props {
        name: string;
        label: string;
        value: string | number;
        type?: "text" | "email" | "password";
        placeholder?: string;
        autocomplete?: string;
        rows?: number;
        error?: string;
        helper?: string;
        class?: string;
    }

    let {
        name,
        label,
        value = $bindable(),
        type = "text",
        placeholder = "",
        autocomplete = "off",
        rows = 0,
        error = "",
        helper = "\x80",
        class: className = "",
    }: Props = $props();

    function typeAction(node: HTMLInputElement): void {
        node.type = type;
    }
</script>

<div class={className}>
    <label for={name} class="block text-sm font-medium leading-6">{label}</label>
    <div class="mt-2">
        {#if rows === 0}
            <input
                use:typeAction
                bind:value
                id={name}
                {name}
                {placeholder}
                {autocomplete}
                class="block w-full rounded-md border-0 bg-gray-800 px-3 py-1.5 shadow-inset placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6 {error
                    ? 'outline outline-1 outline-red-600'
                    : ''}"
                aria-invalid={!!error}
                aria-describedby="{name}-description"
            />
        {:else}
            <textarea
                bind:value
                id={name}
                {name}
                {placeholder}
                {rows}
                class="block w-full rounded-md border-0 bg-gray-800 px-3 py-1.5 shadow-inset placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6 {error
                    ? 'outline outline-1 outline-red-600'
                    : ''}"
                aria-invalid={!!error}
                aria-describedby="{name}-description"
            />
        {/if}
    </div>
    <p id="{name}-description" class="mb-2 mt-0 text-sm leading-6 {error ? 'text-red-600' : 'text-gray-500'}">
        {error || helper}
    </p>
</div>
