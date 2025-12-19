<script lang="ts">
    import { emit, once } from "@tauri-apps/api/event";
    import Button from "$lib/components/ui/button/button.svelte";
    import Input from "$lib/components/ui/input/input.svelte";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";

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
</script>

<main class="container">
    <h1 class="text-lg font-bold mb-2">authentication required</h1>
    <form class="gap-2 flex flex-col" onsubmit={unlockVault}>
        <Input
            type="password"
            placeholder="enter your vault password"
            bind:value={passwordInput}
            autofocus
            required
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
