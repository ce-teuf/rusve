import { apiFetch } from "$lib/mobile/api";
import type { Note__Output } from "$lib/proto/proto/Note";
import type { PageLoad } from "./$types";

type NoteData = {note: Note__Output | null; error?: string };

export const load: PageLoad = async ({ data, params }) => {
    if (data && "note" in data) return data;

    const result = await apiFetch<{ 
        note: Note__Output
    }>(`/api/notes/${params.noteId}`);

    if (!result.ok) return { note: null, error: result.error } as NoteData;

    return result.data as NoteData;
};
