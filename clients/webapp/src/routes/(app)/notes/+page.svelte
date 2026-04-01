<script lang="ts">
    import { toast } from "$lib/ui/toast";
    import Input from "$lib/form/Input.svelte";
    import Button from "$lib/form/Button.svelte";
    import { enhance } from "$app/forms";
    import { extractError } from "$lib/errors";
    import SaveIcon from "$lib/icons/SaveIcon.svelte";
    import Pagination from "$lib/ui/Pagination.svelte";
    import { preloadData, pushState, goto } from "$app/navigation";
    import { page } from "$app/state";
    import Drawer from "$lib/ui/Drawer.svelte";
    import NotePage from "./[noteId]/+page.svelte";
    import type { PageData, ActionData } from "./$types";
    import type { PageData as NotePageData } from "./[noteId]/$types";
    import type { NoteResponse__Output } from "$lib/proto/proto/NoteResponse";
    import type { Note__Output } from "$lib/proto/proto/Note";


    // let { data, form }: { data: PageData; form: ActionData } = $props();
    let { data, form }: { 
        data: PageData & {
            notes: Note__Output[];
            total: number;
            pageSize: number;
            error?: string;
        }; 
        form: ActionData 
    } = $props();

    $effect(() => {
        if (form?.error || data?.error) {
            toast.error("Error", form?.error || data?.error || "Unknown error");
        }
    });

    let title = $state("");
    let content = $state("");
    let loading = $state(false);

    async function onDetails(e: MouseEvent & { currentTarget: EventTarget & HTMLAnchorElement }): Promise<void> {
        if (e.metaKey || innerWidth < 640) return;
        e.preventDefault();
        const { href } = e.currentTarget;
        const result = await preloadData(href);
        if (result["type"] === "loaded" && result["status"] === 200) {
            pushState(href, { noteDrawer: result["data"] as NotePageData, open: true });
        } else {
            goto(href);
        }
    }
</script>

{#if page.state.open}
    <Drawer open={page.state.open} close={() => history.back()} title="Note details">
        <NotePage isModal data={page.state.noteDrawer} {form} />
    </Drawer>
{/if}

<form
    class="max-w-2xl"
    action="?/insert"
    method="post"
    use:enhance={() => {
        const timeout = setTimeout(() => { loading = true; }, 100);
        return async ({ result, update }) => {
            if (result.type === "success") toast.success("Success", "Note created");
            clearTimeout(timeout);
            loading = false;
            await update();
        };
    }}
>
    <div class="space-y-12">
        <div>
            <h2 class="flex items-center gap-2 text-base font-semibold leading-7 text-gray-50">New note</h2>
            <p class="mt-1 text-sm leading-6 text-gray-200">Create a new note.</p>
        </div>
        <div class="mt-10 grid grid-cols-1 gap-x-6 sm:grid-cols-6">
            <div class="sm:col-span-4">
                <Input name="title" label="Title" bind:value={title} error={extractError(form?.fields, "title")} />
            </div>
            <div class="col-span-full">
                <Input name="content" label="Content" bind:value={content} error={extractError(form?.fields, "content")} rows={3} helper="Max 1000 characters" />
            </div>
            <div class="col-span-full flex justify-end">
                <Button type="submit" {loading}>
                    {#snippet icon()}<SaveIcon />{/snippet}
                    Save
                </Button>
            </div>
        </div>
    </div>
</form>

<div class="mt-10 sm:flex sm:items-center">
    <div class="sm:flex-auto">
        <h1 class="text-base font-semibold leading-6 text-gray-50">Notes</h1>
        <p class="mt-2 text-sm leading-6 text-gray-200">List of notes you have created.</p>
    </div>
</div>
<div class="mt-8 flow-root max-w-7xl">
    <div class="overflow-x-auto overflow-y-hidden">
        <div class="inline-block min-w-full align-middle">
            <table class="min-w-full divide-y divide-gray-600">
                <thead>
                    <tr>
                        <th scope="col" class="py-3 pl-4 pr-3 text-left text-xs uppercase tracking-wide text-gray-500 sm:pl-0">Title</th>
                        <th scope="col" class="px-3 py-3 text-left text-xs uppercase tracking-wide text-gray-500">Content</th>
                        <th scope="col" class="px-3 py-3 text-left text-xs uppercase tracking-wide text-gray-500">Profile name</th>
                        <th scope="col" class="px-3 py-3 text-left text-xs uppercase tracking-wide text-gray-500">Created</th>
                        <th scope="col" class="px-3 py-3 text-left text-xs uppercase tracking-wide text-gray-500">Updated</th>
                        <th scope="col" class="relative py-3 pl-3 pr-4 sm:pr-0"><span class="sr-only">Edit</span></th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-600 bg-gray-900">
                    {#each data.notes as note}
                        {#if note.note === null}
                            <tr>
                                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-50 sm:pl-0" colspan="5">No notes found</td>
                            </tr>
                        {:else}
                            <tr>
                                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-50 sm:pl-0">{note.note.title}</td>
                                <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-200">{note.note.content}</td>
                                <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-200">{note.profile?.name || "Unknown"}</td>
                                <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-200">{note.note.created}</td>
                                <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-200">{note.note.updated}</td>
                                <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0">
                                    <a href="/notes/{note.note.id}" class="mr-4 text-indigo-600 hover:text-indigo-900" onclick={(e) => onDetails(e)}>
                                        Edit<span class="sr-only">, {note.note.title}</span>
                                    </a>
                                </td>
                            </tr>
                        {/if}
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
    <Pagination total={data.total} pageSize={data.pageSize} />
</div>
