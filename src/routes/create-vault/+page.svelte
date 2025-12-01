<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    type Key = {
        id: string;
        name: string;
        key_type: "public" | "private";
        date_created: Date;
    };

    let passwordInput: HTMLInputElement | null = null;

    let error = $state("");

    async function createVault(event: Event) {
        event.preventDefault();
        if (!passwordInput) {
            error = "password input field could not be found";
            return;
        }
        error = await invoke("create_vault", { password: passwordInput.value });
        if (!error) {
            passwordInput.value =
                "don't read the password please that would not be nice and i really don't think you should do that";
            await invoke("load_vault");
            window.location.href = "/home";
        }
    }

    // let isFirstOpenRes: Promise<boolean> = invoke("is_first_open");
</script>

<main class="container">
    <p>welcome :P</p>
    <p>let's make a vault to store your private keys!</p>
    <h1>choose a password</h1>
    <input bind:this={passwordInput} type="password" />
    <button onclick={createVault}>create</button>
</main>
