export function generateId(): string {
    const timestamp = new Date().getTime().toString(36);
    const random = Math.random().toString(36).substring(2, 5);
    return timestamp + random;
}

export function checkElement(element: Element | null | undefined): HTMLElement | undefined {
    if (!element || !(element instanceof HTMLElement)) return undefined;
    return element;
}

export function getFormValue(form: FormData, key: string): string {
    const value = form.get(key);
    if (!value || typeof value !== "string") return "";
    return value;
}
