<script lang="ts">
    import { enhance } from "$app/forms";
    import Button from "$lib/form/Button.svelte";
    import Input from "$lib/form/Input.svelte";
    import type { PageData, ActionData } from "./$types";

    let { data, form }: { data: PageData; form: ActionData } = $props();

    let name = $state(data.source.name);
    let source_url = $state(data.source.source_url);
    let source_type = $state(data.source.source_type);
    let integration_mode = $state(data.source.integration_mode ?? "MANUAL");
    let auto_schedule = $state(data.source.auto_schedule ?? "");
    let field_rules = $state(data.source.field_rules ?? "[]");
    let active = $state(data.source.active ?? true);
    let loading = $state(false);
    let deleteLoading = $state(false);

    let rulesError = $derived(() => {
        try { JSON.parse(field_rules); return ""; }
        catch { return "Invalid JSON"; }
    });
</script>

<div class="min-h-screen bg-gray-900 text-white p-6">
    <div class="max-w-2xl mx-auto space-y-6">

        <div>
            <a href="/admin/scraper/sources" class="text-xs text-gray-500 hover:text-gray-300">← Sources</a>
            <h1 class="text-2xl font-bold mt-1">{data.source.name}</h1>
            <p class="text-gray-400 text-sm">{data.source.source_url}</p>
        </div>

        <form
            method="POST"
            action="?/update"
            class="bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-5"
            use:enhance={() => {
                loading = true;
                return async ({ update }) => {
                    await update({ reset: false });
                    loading = false;
                };
            }}
        >
            <h2 class="font-semibold text-gray-200">Configuration</h2>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <Input name="name" label="Name" bind:value={name} />
                <Input name="source_type" label="Type" bind:value={source_type} placeholder="article, product, ..." />
            </div>

            <Input name="source_url" label="URL" type="text" bind:value={source_url} />

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-300 mb-1">Integration mode</label>
                    <select
                        name="integration_mode"
                        bind:value={integration_mode}
                        class="w-full bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-white text-sm"
                    >
                        <option value="MANUAL">MANUAL — admin approval</option>
                        <option value="AUTO">AUTO — automatic push</option>
                    </select>
                </div>
                <Input
                    name="auto_schedule"
                    label="Cron schedule"
                    bind:value={auto_schedule}
                    placeholder="0 2 * * *"
                    helper={integration_mode === "AUTO" ? "Required for AUTO mode" : "Only used in AUTO mode"}
                />
            </div>

            <div class="flex items-center gap-3">
                <input
                    type="checkbox"
                    id="active"
                    name="active"
                    value="true"
                    checked={active}
                    onchange={(e) => (active = (e.target as HTMLInputElement).checked)}
                    class="w-4 h-4 accent-indigo-500"
                />
                <label for="active" class="text-sm text-gray-300">Active (enabled in scheduler)</label>
            </div>

            <div>
                <label class="block text-sm font-medium text-gray-300 mb-1">
                    field_rules
                    <span class="text-gray-500 font-normal"> — JSON array of validation constraints</span>
                </label>
                <textarea
                    name="field_rules"
                    bind:value={field_rules}
                    rows="8"
                    spellcheck="false"
                    class="w-full bg-gray-700 border {rulesError() ? 'border-red-500' : 'border-gray-600'} rounded-md px-3 py-2 text-white text-xs font-mono resize-y"
                ></textarea>
                {#if rulesError()}
                    <p class="text-xs text-red-400 mt-1">{rulesError()}</p>
                {:else}
                    <p class="text-xs text-gray-500 mt-1">
                        Example: <code>[&#123;"field":"title","required":true&#125;,&#123;"field":"url","required":true,"format":"url"&#125;]</code>
                    </p>
                {/if}
            </div>

            {#if form?.error}
                <p class="text-sm text-red-400">{form.error}</p>
            {/if}
            {#if form?.success}
                <p class="text-sm text-green-400">Saved — scheduler reloaded.</p>
            {/if}

            <Button type="submit" {loading}>Save</Button>
        </form>

        <!-- Danger zone -->
        <div class="bg-gray-800 rounded-lg p-6 border border-red-900 space-y-3">
            <h2 class="font-semibold text-red-400">Danger zone</h2>
            <p class="text-sm text-gray-400">Deleting a source does not delete its jobs or items (for audit purposes).</p>
            <form method="POST" action="?/delete" use:enhance={() => { deleteLoading = true; return async ({ update }) => { await update(); deleteLoading = false; }; }}>
                <Button type="submit" loading={deleteLoading} class="bg-red-700 hover:bg-red-600">Delete source</Button>
            </form>
        </div>

    </div>
</div>
