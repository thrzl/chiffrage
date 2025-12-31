<script lang="ts">
    import * as AlertDialog from "$lib/components/ui/alert-dialog/index";
    import PasswordBox from "./PasswordBox.svelte";
    import SlideAlert from "./SlideAlert.svelte";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { OctagonXIcon } from "@lucide/svelte";
    import { once, emit, listen } from "@tauri-apps/api/event";
    import type { SvelteComponent } from "svelte";

    let {
        description = "this action requires authentication. please enter your vault password to continue.",
    }: {
        description?: string;
    } = $props();
    let open = $state(false);
    let alert: { title: string; description: string } | undefined =
        $state(undefined);
    let passwordInput = $state("");
    let isLoading = $state(false);
    let passwordElement: SvelteComponent | undefined;
    listen("auth-start", () => {
        passwordElement?.focus();
        open = true;
    });
    listen("vault-status-update", () => {
        isLoading = false;
        open = false;
        passwordInput = "";
    });
    async function cancelAuth() {
        await emit("auth-cancel", {});
        open = false;
    }
    async function unlockVault(event: Event | undefined) {
        event?.preventDefault();
        alert = undefined;
        if (!passwordInput) return;
        await once("auth-response", (e) => {
            isLoading = false;
            let authSuccessful = e.payload as boolean;
            if (authSuccessful) {
                open = false;
                return;
            }
            alert = {
                title: "authentication failed",
                description: "",
            };
        });
        isLoading = true;
        await emit("authenticate", passwordInput);
    }
    document.addEventListener("keypress", async (e) => {
        if (open && e.key === "Escape") open = false;
    });
</script>

<AlertDialog.Root bind:open>
    <AlertDialog.Content class="z-999">
        <AlertDialog.Header>
            <AlertDialog.Title>unlock vault</AlertDialog.Title>
            <AlertDialog.Description>
                {description}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <form class="flex flex-col" onsubmit={unlockVault}>
            <PasswordBox
                bind:this={passwordElement}
                placeholder="enter your vault password..."
                bind:password={passwordInput}
                showMeter={false}
                showGenerate={false}
                autofocus
                style="margin-bottom: 0.5rem"
            />
            <SlideAlert
                bind:alert
                class="border-red-800 bg-red-950 my-0"
                icon={OctagonXIcon}
            />
        </form>
        <AlertDialog.Footer>
            <AlertDialog.Cancel onclick={cancelAuth}>cancel</AlertDialog.Cancel>
            <AlertDialog.Action
                onclick={unlockVault}
                disabled={isLoading || passwordInput === ""}
                >{#if isLoading}<Spinner />processing...{:else}unlock vault{/if}</AlertDialog.Action
            >
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
