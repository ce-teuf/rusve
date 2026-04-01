<script lang="ts">
    import { enhance } from "$app/forms";
    import Button from "$lib/form/Button.svelte";
    import Input from "$lib/form/Input.svelte";
    import LogoIcon from "$lib/icons/LogoIcon.svelte";
    import type { ActionData } from "./$types";

    let { form }: { form: ActionData } = $props();

    let email = $state("");
    $effect(() => { email = form?.email ?? ""; });
    let password = $state("");
    let confirm = $state("");
    let loading = $state(false);

    const requirements = [
        { id: "length", label: "At least 12 characters", test: (p: string) => p.length >= 12 },
        { id: "digit", label: "At least 2 digits", test: (p: string) => (p.match(/\d/g) || []).length >= 2 },
        { id: "special", label: "At least 1 special character (!@#$%...)", test: (p: string) => /[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(p) },
        { id: "upper", label: "At least 1 uppercase letter", test: (p: string) => /[A-Z]/.test(p) },
        { id: "lower", label: "At least 1 lowercase letter", test: (p: string) => /[a-z]/.test(p) },
    ];

    let met = $derived(
        Object.fromEntries(requirements.map(r => [r.id, r.test(password)]))
    );
    let allMet = $derived(Object.values(met).every(Boolean));
</script>

<main class="flex min-h-full flex-col justify-center bg-gray-900 px-6 py-12 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-sm">
        <div class="mx-auto h-10 w-10 text-indigo-600"><LogoIcon /></div>
        <h2 class="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-white">Create your account</h2>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-sm">
        <form
            method="POST"
            class="space-y-2"
            use:enhance={() => {
                loading = true;
                return async ({ update }) => {
                    await update({ reset: false });
                    loading = false;
                };
            }}
        >
            <Input
                name="email"
                label="Email"
                bind:value={email}
                type="email"
                autocomplete="email"
                placeholder="you@example.com"
            />
            <Input
                name="password"
                label="Password"
                bind:value={password}
                type="password"
                autocomplete="new-password"
                placeholder="••••••••••••"
            />
            <div class="text-xs text-gray-400 space-y-1 mb-3">
                <p class="font-medium text-gray-300 mb-2">Password requirements:</p>
                <ul class="space-y-1">
                    {#each requirements as req}
                        <li class="flex items-center gap-2">
                            <span class:text-green-400={met[req.id]} class:text-gray-500={!met[req.id]}>
                                {met[req.id] ? "✓" : "○"}
                            </span>
                            <span class:text-green-400={met[req.id]} class:text-gray-500={!met[req.id]}>
                                {req.label}
                            </span>
                        </li>
                    {/each}
                </ul>
            </div>
            <Input
                name="confirm_password"
                label="Confirm password"
                bind:value={confirm}
                type="password"
                autocomplete="new-password"
                placeholder="••••••••••••"
            />
            {#if confirm.length > 0}
                <p class="text-xs {password === confirm ? 'text-green-400' : 'text-red-400'}">
                    {password === confirm ? "✓ Passwords match" : "✗ Passwords do not match"}
                </p>
            {/if}

            {#if form?.error}
                <p class="text-sm text-red-400">{form.error}</p>
            {/if}

            <Button type="submit" {loading} disabled={!allMet || password !== confirm} class="w-full mt-4">
                Create account
            </Button>
        </form>

        <p class="mt-6 text-center text-sm text-gray-400">
            Already have an account?
            <a href="/auth" class="font-medium text-indigo-400 hover:text-indigo-300">Sign in</a>
        </p>
    </div>
</main>