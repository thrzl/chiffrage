<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";
    import { onMount } from "svelte";

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
    <p>welcome back</p>
    <input type="password" placeholder="enter your vault password" />
    <button type="submit" onclick={unlockVault}>unlock vault</button>
    <p style:color="red">{error}</p>
</main>
