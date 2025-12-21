<script lang="ts">
    import { emit, once } from "@tauri-apps/api/event";
    import Button from "$lib/components/ui/button/button.svelte";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import PasswordBox from "../../components/PasswordBox.svelte";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

    let passwordInput: string = $state("");
    let isLoading: boolean = $state(false);

    let error = $state("");

    async function unlockVault(event: Event | undefined) {
        event?.preventDefault();
        if (!passwordInput) return;
        await once("auth-error", (e) => {
            isLoading = false;
            error = e.payload as string;
        });
        isLoading = true;
        await emit("authenticate", passwordInput);
    }
    document.addEventListener("keypress", async (e) => {
        if (e.key === "Escape") await getCurrentWebviewWindow().close();
    });
</script>

<main class="container">
    <h1 class="text-lg font-bold mb-2">authentication required</h1>
    <form class="gap-2 flex flex-col" onsubmit={unlockVault}>
        <PasswordBox
            placeholder="enter your vault password..."
            bind:password={passwordInput}
            showMeter={false}
            showGenerate={false}
            autofocus
        />
        <Button
            class="w-full"
            type="submit"
            onclick={unlockVault}
            disabled={isLoading || passwordInput === ""}
            >{#if isLoading}<Spinner />processing...{:else}unlock vault{/if}</Button
        >
    </form>
    <p style:color="red">{error}</p>
</main>
