<script lang="ts">
    import type { PageData } from "./$types";
    import type { JobResponse__Output } from "$lib/proto/proto/JobResponse";

    let { data }: { data: PageData } = $props();

    const statusColor: Record<string, string> = {
        RUNNING: "text-blue-400",
        DONE: "text-green-400",
        FAILED: "text-red-400",
    };

    function formatDate(iso: string) {
        if (!iso) return "—";
        return new Date(iso).toLocaleString();
    }

    function jobStatus(job: JobResponse__Output) {
        return job.job?.status ?? "";
    }
</script>

<div class="min-h-screen bg-gray-900 text-white p-6">
    <div class="max-w-6xl mx-auto space-y-8">

        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold">Scraper — Dashboard</h1>
                <p class="text-gray-400 text-sm mt-1">{data.sources.length} sources · {data.jobs.length} recent jobs</p>
            </div>
            <a href="/admin/scraper/sources" class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 rounded-md text-sm font-medium">
                Manage sources
            </a>
        </div>

        <!-- Sources summary -->
        {#if data.sources.length > 0}
            <section>
                <h2 class="text-lg font-semibold mb-3 text-gray-200">Sources</h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                    {#each data.sources as source}
                        <a href="/admin/scraper/sources/{source.id}" class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750 border border-gray-700">
                            <div class="flex items-start justify-between gap-2">
                                <div class="min-w-0">
                                    <p class="font-medium truncate">{source.name}</p>
                                    <p class="text-xs text-gray-400 truncate mt-0.5">{source.source_url}</p>
                                </div>
                                <span class="shrink-0 text-xs px-2 py-0.5 rounded-full {source.integration_mode === 'AUTO' ? 'bg-green-900 text-green-300' : 'bg-orange-900 text-orange-300'}">
                                    {source.integration_mode}
                                </span>
                            </div>
                            {#if source.integration_mode === 'AUTO' && source.auto_schedule}
                                <p class="text-xs text-gray-500 mt-2">⏱ {source.auto_schedule}</p>
                            {/if}
                            <div class="flex items-center gap-2 mt-2">
                                <span class="w-2 h-2 rounded-full {source.active ? 'bg-green-500' : 'bg-gray-600'}"></span>
                                <span class="text-xs text-gray-400">{source.active ? 'Active' : 'Inactive'}</span>
                            </div>
                        </a>
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Jobs table -->
        <section>
            <h2 class="text-lg font-semibold mb-3 text-gray-200">Recent jobs</h2>
            {#if data.jobs.length === 0}
                <p class="text-gray-500 text-sm">No jobs yet. Run <code class="text-indigo-400">python scrapers/run.py</code> to create one.</p>
            {:else}
                <div class="overflow-x-auto rounded-lg border border-gray-700">
                    <table class="w-full text-sm">
                        <thead class="bg-gray-800 text-gray-400 text-xs uppercase">
                            <tr>
                                <th class="px-4 py-3 text-left">Source</th>
                                <th class="px-4 py-3 text-left">Type</th>
                                <th class="px-4 py-3 text-left">Status</th>
                                <th class="px-4 py-3 text-right">Items</th>
                                <th class="px-4 py-3 text-left">Date</th>
                                <th class="px-4 py-3"></th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-700">
                            {#each data.jobs as jr}
                                <tr class="hover:bg-gray-800/50">
                                    <td class="px-4 py-3 font-medium max-w-xs truncate">
                                        {jr.source_name || jr.job?.source_url || '—'}
                                    </td>
                                    <td class="px-4 py-3 text-gray-400">{jr.job?.source_type}</td>
                                    <td class="px-4 py-3">
                                        <span class="font-medium {statusColor[jobStatus(jr)] ?? 'text-gray-400'}">
                                            {jobStatus(jr)}
                                        </span>
                                        {#if jr.job?.error}
                                            <p class="text-xs text-red-400 truncate max-w-xs">{jr.job.error}</p>
                                        {/if}
                                    </td>
                                    <td class="px-4 py-3 text-right tabular-nums">{jr.job?.item_count ?? 0}</td>
                                    <td class="px-4 py-3 text-gray-400">{formatDate(jr.job?.created ?? '')}</td>
                                    <td class="px-4 py-3 text-right">
                                        <a href="/admin/scraper/{jr.job?.id}" class="text-indigo-400 hover:text-indigo-300 text-xs">
                                            View →
                                        </a>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        </section>

    </div>
</div>
