import { writable } from "svelte/store";
import { generateId } from "$lib/utils";
import type { Toast } from "$lib/types";

export const toastStore = writable<Toast[]>([]);

function showToast(t: Toast): void {
    toastStore.update((toasts) => [...toasts, t]);
    setTimeout(() => {
        toastStore.update((toasts) => toasts.filter((x) => x.id !== t.id));
    }, t.duration);
}

export const toast = {
    success: (title: string, description: string) =>
        showToast({ id: generateId(), title, description, type: "success", duration: 4000 }),
    error: (title: string, description: string) =>
        showToast({ id: generateId(), title, description, type: "error", duration: 4000 }),
    warning: (title: string, description: string) =>
        showToast({ id: generateId(), title, description, type: "warning", duration: 4000 }),
    info: (title: string, description: string) =>
        showToast({ id: generateId(), title, description, type: "info", duration: 4000 }),
};
