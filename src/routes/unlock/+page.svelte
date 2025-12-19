<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";
    import Button from "$lib/components/ui/button/button.svelte";
    import Input from "$lib/components/ui/input/input.svelte";

    let passwordInput: HTMLInputElement | null = null;

    let error = $state("");

    async function unlockVault(event: Event) {
        event.preventDefault();
        // error = invokeError || "";
        emit(
            "authenticate",
            (document.querySelector("input") as HTMLInputElement).value,
        );
    }
</script>

<main class="container">
    <h1 class="text-lg font-bold mb-2">authentication required</h1>
    <form class="gap-2 flex flex-col">
        <Input type="password" placeholder="enter your vault password" />
        <Button class="w-full" type="submit" onclick={unlockVault}
            >unlock vault</Button
        >
    </form>
    <p style:color="red">{error}</p>
</main>
