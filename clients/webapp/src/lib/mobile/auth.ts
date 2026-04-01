/**
 * Token management for Capacitor mobile.
 * On native (Android/iOS): uses @capacitor/preferences (persistent, secure storage).
 * On web: falls back to an in-memory variable (the cookie handles auth instead).
 */

const TOKEN_KEY = "rusve_token";

// In-memory fallback for web (SSR-safe: only runs in browser)
let _memToken = "";

function isCapacitor(): boolean {
    return typeof window !== "undefined" && !!(window as { Capacitor?: { isNative?: boolean } }).Capacitor?.isNative;
}

export async function setToken(token: string): Promise<void> {
    if (isCapacitor()) {
        const { Preferences } = await import("@capacitor/preferences");
        await Preferences.set({ key: TOKEN_KEY, value: token });
    } else {
        _memToken = token;
    }
}

export async function getToken(): Promise<string> {
    if (isCapacitor()) {
        const { Preferences } = await import("@capacitor/preferences");
        const { value } = await Preferences.get({ key: TOKEN_KEY });
        return value ?? "";
    }
    return _memToken;
}

export async function clearToken(): Promise<void> {
    if (isCapacitor()) {
        const { Preferences } = await import("@capacitor/preferences");
        await Preferences.remove({ key: TOKEN_KEY });
    } else {
        _memToken = "";
    }
}
