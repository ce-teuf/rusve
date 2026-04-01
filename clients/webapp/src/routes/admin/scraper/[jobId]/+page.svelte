<script lang="ts">
    import { enhance } from "$app/forms";
    import type { PageData, ActionData } from "./$types";
    import type { Item__Output } from "$lib/proto/proto/Item";

    let { data, form }: { data: PageData; form: ActionData } = $props();

    const STATUSES = ["", "PENDING", "VALID", "INVALID", "APPROVED", "REJECTED", "PUSHED"];
    const statusColor: Record<string, string> = {
        PENDING:  "bg-gray-700 text-gray-300",
        VALID:    "bg-blue-900 text-blue-300",
        INVALID:  "bg-red-900 text-red-300",
        APPROVED: "bg-green-900 text-green-300",
        REJECTED: "bg-gray-800 text-gray-500",
        PUSHED:   "bg-purple-900 text-purple-300",
    };

    let expanded = $state<string | null>(null);
    let actionLoading = $state<string | null>(null);

    function formatJson(raw: string) {
        try { return JSON.stringify(JSON.parse(raw), null, 2); }
        catch { return raw; }
    }

    function countByStatus(items: Item__Output[], status: string) {
        return items.filter((i) => i.validation_status === status).length;
    }

    const approvedCount = $derived(countByStatus(data.items, "APPROVED"));
    const validCount    = $derived(countByStatus(data.items, "VALID"));
</script>

<div class="min-h-screen bg-gray-900 text-white p-6">
    <div class="max-w-5xl mx-auto space-y-6">

        <!-- Header -->
        <div>
            <a href="/admin/scraper" class="text-xs text-gray-500 hover:text-gray-300">← Dashboard</a>
            <h1 class="text-2xl font-bold mt-1">Job detail</h1>
            <div class="flex flex-wrap gap-4 mt-2 text-sm text-gray-400">
                <span>{data.job.source_url}</span>
                <span class="text-gray-600">·</span>
                <span>{data.job.source_type}</span>
                <span class="text-gray-600">·</span>
                <span class="font-medium {data.job.status === 'DONE' ? 'text-green-400' : data.job.status === 'FAILED' ? 'text-red-400' : 'text-blue-400'}">
                    {data.job.status}
                </span>
                <span class="text-gray-600">·</span>
                <span>{data.job.item_count} items</span>
            </div>
            {#if data.job.error}
                <p class="text-sm text-red-400 mt-1">{data.job.error}</p>
            {/if}
        </div>

        <!-- Feedback -->
        {#if form?.error}
            <p class="text-sm text-red-400">{form.error}</p>
        {/if}
        {#if form?.approvedCount !== undefined}
            <p class="text-sm text-green-400">{form.approvedCount} items approved.</p>
        {/if}
        {#if form?.pushedCount !== undefined}
            <p class="text-sm text-purple-400">{form.pushedCount} items pushed to db_data.</p>
        {/if}

        <!-- Batch actions -->
        <div class="flex flex-wrap gap-3">
            {#if validCount > 0}
                <form method="POST" action="?/approveAllValid" use:enhance={() => { actionLoading = "approveAll"; return async ({ update }) => { await update({ reset: false }); actionLoading = null; }; }}>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-green-800 hover:bg-green-700 rounded-md text-sm font-medium disabled:opacity-50"
                        disabled={actionLoading === "approveAll"}
                    >
                        {actionLoading === "approveAll" ? "..." : `Approve all valid (${validCount})`}
                    </button>
                </form>
            {/if}
            {#if approvedCount > 0}
                <form method="POST" action="?/pushApproved" use:enhance={() => { actionLoading = "push"; return async ({ update }) => { await update({ reset: false }); actionLoading = null; }; }}>
                    <button
                        type="submit"
                        class="px-4 py-2 bg-purple-800 hover:bg-purple-700 rounded-md text-sm font-medium disabled:opacity-50"
                        disabled={actionLoading === "push"}
                    >
                        {actionLoading === "push" ? "..." : `Push approved (${approvedCount}) → db_data`}
                    </button>
                </form>
            {/if}
        </div>

        <!-- Status filter -->
        <div class="flex flex-wrap gap-2">
            {#each STATUSES as s}
                <a
                    href="?{s ? `status=${s}` : ''}"
                    class="px-3 py-1 rounded-full text-xs font-medium border {data.statusFilter === s ? 'border-indigo-500 bg-indigo-900 text-indigo-200' : 'border-gray-700 text-gray-400 hover:border-gray-500'}"
                >
                    {s || "All"} ({s ? countByStatus(data.items, s) : data.items.length})
                </a>
            {/each}
        </div>

        <!-- Items list -->
        {#if data.items.length === 0}
            <p class="text-gray-500 text-sm">No items{data.statusFilter ? ` with status ${data.statusFilter}` : ""}.</p>
        {:else}
            <div class="space-y-2">
                {#each data.items as item}
                    <div class="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
                        <!-- Item header -->
                        <div class="flex items-center justify-between px-4 py-3 gap-3">
                            <div class="flex items-center gap-3 min-w-0">
                                <button
                                    onclick={() => (expanded = expanded === item.id ? null : item.id)}
                                    class="text-gray-400 hover:text-white text-xs shrink-0"
                                >
                                    {expanded === item.id ? "▾" : "▸"}
                                </button>
                                <span class="text-xs px-2 py-0.5 rounded-full {statusColor[item.validation_status ?? ''] ?? 'bg-gray-700 text-gray-400'}">
                                    {item.validation_status}
                                </span>
                                <span class="text-xs text-gray-500 font-mono truncate">{item.id}</span>
                            </div>

                            <!-- Per-item actions -->
                            <div class="flex items-center gap-2 shrink-0">
                                {#if item.validation_status !== "APPROVED" && item.validation_status !== "PUSHED" && item.validation_status !== "REJECTED"}
                                    <form method="POST" action="?/approve" use:enhance={() => { actionLoading = `approve-${item.id}`; return async ({ update }) => { await update({ reset: false }); actionLoading = null; }; }}>
                                        <input type="hidden" name="id" value={item.id} />
                                        <button class="text-xs text-green-400 hover:text-green-300 disabled:opacity-50" disabled={actionLoading === `approve-${item.id}`}>
                                            Approve
                                        </button>
                                    </form>
                                {/if}
                                {#if item.validation_status !== "REJECTED" && item.validation_status !== "PUSHED"}
                                    <form method="POST" action="?/reject" use:enhance={() => { actionLoading = `reject-${item.id}`; return async ({ update }) => { await update({ reset: false }); actionLoading = null; }; }}>
                                        <input type="hidden" name="id" value={item.id} />
                                        <button class="text-xs text-red-400 hover:text-red-300 disabled:opacity-50" disabled={actionLoading === `reject-${item.id}`}>
                                            Reject
                                        </button>
                                    </form>
                                {/if}
                                {#if item.pushed_at}
                                    <span class="text-xs text-gray-500">pushed</span>
                                {/if}
                            </div>
                        </div>

                        <!-- Validation errors -->
                        {#if item.validation_errors && item.validation_errors !== "[]"}
                            <div class="px-4 pb-2">
                                {#each (JSON.parse(item.validation_errors) as string[]) as err}
                                    <p class="text-xs text-red-400">• {err}</p>
                                {/each}
                            </div>
                        {/if}

                        <!-- Expanded raw_data -->
                        {#if expanded === item.id}
                            <div class="border-t border-gray-700 p-4">
                                <pre class="text-xs text-gray-300 font-mono overflow-x-auto whitespace-pre-wrap">{formatJson(item.raw_data ?? "{}")}</pre>
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}

    </div>
</div>
