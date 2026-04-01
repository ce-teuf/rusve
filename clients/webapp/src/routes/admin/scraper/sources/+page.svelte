<script lang="ts">
    import { enhance } from "$app/forms";
    import Button from "$lib/form/Button.svelte";
    import Input from "$lib/form/Input.svelte";
    import type { PageData, ActionData } from "./$types";

    let { data, form }: { data: PageData; form: ActionData } = $props();

    let showCreate = $state(false);
    let loading = $state(false);
</script>

<div class="min-h-screen bg-gray-900 text-white p-6">
    <div class="max-w-4xl mx-auto space-y-6">

        <div class="flex items-center justify-between">
            <div>
                <a href="/admin/scraper" class="text-xs text-gray-500 hover:text-gray-300">← Dashboard</a>
                <h1 class="text-2xl font-bold mt-1">Scrape sources</h1>
            </div>
            <button
                onclick={() => (showCreate = !showCreate)}
                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 rounded-md text-sm font-medium"
            >
                {showCreate ? "Cancel" : "+ Add source"}
            </button>
        </div>

        <!-- Create form -->
        {#if showCreate}
            <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <h2 class="text-lg font-semibold mb-4">New source</h2>
                <form
                    method="POST"
                    action="?/create"
                    class="space-y-4"
                    use:enhance={() => {
                        loading = true;
                        return async ({ update }) => {
                            await update();
                            loading = false;
                        };
                    }}
                >
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <Input name="name" label="Name" placeholder="Mon Blog" />
                        <Input name="source_type" label="Type" placeholder="article, product, ..." />
                    </div>
                    <Input name="source_url" label="URL" type="text" placeholder="https://example.com/feed" />
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-1">Integration mode</label>
                            <select name="integration_mode" class="w-full bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-white text-sm">
                                <option value="MANUAL">MANUAL — admin approval</option>
                                <option value="AUTO">AUTO — automatic push</option>
                            </select>
                        </div>
                        <Input name="auto_schedule" label="Cron schedule (AUTO only)" placeholder="0 2 * * *" />
                    </div>
                    {#if form?.error}
                        <p class="text-sm text-red-400">{form.error}</p>
                    {/if}
                    <Button type="submit" {loading}>Create source</Button>
                </form>
            </div>
        {/if}

        <!-- Sources list -->
        {#if data.sources.length === 0}
            <p class="text-gray-500 text-sm">No sources yet.</p>
        {:else}
            <div class="space-y-2">
                {#each data.sources as source}
                    <a
                        href="/admin/scraper/sources/{source.id}"
                        class="flex items-center justify-between bg-gray-800 hover:bg-gray-750 border border-gray-700 rounded-lg px-5 py-4"
                    >
                        <div class="min-w-0">
                            <div class="flex items-center gap-3">
                                <span class="font-medium">{source.name}</span>
                                <span class="text-xs px-2 py-0.5 rounded-full {source.integration_mode === 'AUTO' ? 'bg-green-900 text-green-300' : 'bg-orange-900 text-orange-300'}">
                                    {source.integration_mode}
                                </span>
                                {#if !source.active}
                                    <span class="text-xs text-gray-500">inactive</span>
                                {/if}
                            </div>
                            <p class="text-xs text-gray-400 mt-0.5 truncate">{source.source_url}</p>
                            {#if source.integration_mode === 'AUTO' && source.auto_schedule}
                                <p class="text-xs text-gray-500 mt-0.5">⏱ {source.auto_schedule}</p>
                            {/if}
                        </div>
                        <span class="text-gray-500 text-sm ml-4">→</span>
                    </a>
                {/each}
            </div>
        {/if}

    </div>
</div>
