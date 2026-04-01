<script lang="ts">
    import { enhance } from "$app/forms";
    import Button from "$lib/form/Button.svelte";
    import Input from "$lib/form/Input.svelte";
    import LogoIcon from "$lib/icons/LogoIcon.svelte";
    import type { ActionData, PageData } from "./$types";

    let { form, data }: { form: ActionData | null, data: PageData } = $props();

    let email = $state(data.email);
    let code = $state("");
    let loading = $state(false);
    let resendLoading = $state(false);
    let resendMessage = $state("");

    let codeError = $derived(
        code.length > 0 && !/^[A-Z1-9]{6}$/.test(code) ? "Code must be 6 characters (A-Z, 1-9)" : ""
    );
</script>

<main class="flex min-h-full flex-col justify-center bg-gray-900 px-6 py-12 lg:px-8">
    <div class="sm:mx-auto sm:w-full sm:max-w-sm">
        <div class="mx-auto h-10 w-10 text-indigo-600"><LogoIcon /></div>
        <h2 class="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-white">Verify your account</h2>
    </div>

    <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-sm">
        <form
            method="POST"
            action="?/verify"
            class="space-y-6"
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
            <div>
                <label for="code" class="block text-sm font-medium leading-6 text-white">Verification Code</label>
                <div class="mt-2">
                    <input
                        id="code"
                        name="code"
                        type="text"
                        bind:value={code}
                        maxlength="6"
                        placeholder="ABC123"
                        class="block w-full rounded-md border-0 bg-white/5 py-2.5 text-white shadow-sm ring-1 ring-inset ring-white/10 focus:ring-2 focus:ring-inset focus:ring-indigo-500 sm:text-sm sm:leading-6 uppercase tracking-[0.2em] text-center text-lg"
                    />
                </div>
                <p class="mt-2 text-xs text-gray-400">6 characters (letters A-Z, numbers 1-9)</p>
                {#if codeError}
                    <p class="mt-1 text-sm text-red-400">{codeError}</p>
                {/if}
            </div>

            {#if form?.error}
                <p class="text-sm text-red-400">{form.error}</p>
            {/if}

            <Button type="submit" {loading} disabled={code.length !== 6 || !email} class="w-full">
                Verify account
            </Button>
        </form>

        <div class="mt-6 text-center text-sm text-gray-400">
            Didn't receive the code?
            <form method="POST" action="?/resend" use:enhance={() => {
                resendLoading = true;
                resendMessage = "";
                return async ({ result, update }) => {
                    await update({ reset: false });
                    resendLoading = false;
                    if (result.type === "success") resendMessage = "Code resent!";
                };
            }} class="inline">
                <input type="hidden" name="email" value={email} />
                <button type="submit" disabled={resendLoading} class="font-medium text-indigo-400 hover:text-indigo-300 ml-1 disabled:opacity-50">
                    {resendLoading ? "Sending…" : "Resend"}
                </button>
            </form>
            {#if resendMessage}
                <p class="mt-1 text-xs text-green-400">{resendMessage}</p>
            {/if}
        </div>

        <p class="mt-4 text-center text-sm text-gray-400">
            <a href="/auth/register" class="font-medium text-indigo-400 hover:text-indigo-300">Create a new account</a>
        </p>
    </div>
</main>