<script lang="ts">
    import { page } from "$app/state";
    import CreditCardIcon from "$lib/icons/CreditCardIcon.svelte";
    import FileIcon from "$lib/icons/FileIcon.svelte";
    import FileTextIcon from "$lib/icons/FileTextIcon.svelte";
    import HomeIcon from "$lib/icons/HomeIcon.svelte";
    import LogoIcon from "$lib/icons/LogoIcon.svelte";
    import MailIcon from "$lib/icons/MailIcon.svelte";
    import SettingsIcon from "$lib/icons/SettingsIcon.svelte";
    import UserIcon from "$lib/icons/UserIcon.svelte";

    interface Props {
        close?: () => void;
    }

    let { close = () => {} }: Props = $props();

    let current = $derived(page.url.pathname.split("/")[1]);

    function navClass(name: string): string {
        const base = "group flex gap-x-3 rounded-md p-2 text-sm font-semibold leading-6";
        return current === name
            ? `${base} bg-gray-800 text-white`
            : `${base} text-gray-400 hover:bg-gray-800 hover:text-white`;
    }
</script>

<div class="flex grow flex-col gap-y-5 overflow-y-auto bg-black/10 px-6 pb-4 ring-1 ring-white/5">
    <div class="flex h-16 shrink-0 items-center">
        <a onclick={close} href="/dashboard" class="h-8 w-8 text-indigo-600"><LogoIcon /></a>
        <span class="ml-2 select-none text-xl text-gray-50">usve</span>
    </div>
    <nav class="flex flex-1 flex-col">
        <ul role="list" class="flex flex-1 flex-col gap-y-7">
            <li>
                <ul role="list" class="-mx-2 space-y-1">
                    <li><a onclick={close} href="/dashboard" class={navClass("dashboard")}><HomeIcon />Dashboard</a></li>
                    <li><a onclick={close} href="/profile" class={navClass("profile")}><UserIcon />Profile</a></li>
                    <li><a onclick={close} href="/notes" class={navClass("notes")}><FileTextIcon />Notes</a></li>
                    <li><a onclick={close} href="/emails" class={navClass("emails")}><MailIcon />Emails</a></li>
                    <li><a onclick={close} href="/files" class={navClass("files")}><FileIcon />Files</a></li>
                    <li><a onclick={close} href="/subscription" class={navClass("subscription")}><CreditCardIcon />Subscription</a></li>
                </ul>
            </li>
            <li class="mt-auto">
                <a onclick={close} href="/settings" class="-mx-2 {navClass('settings')}"><SettingsIcon />Settings</a>
            </li>
        </ul>
    </nav>
</div>
