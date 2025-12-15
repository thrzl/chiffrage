<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let passwordInput: HTMLInputElement | null = null;

    let error = $state("");

    onMount(async () => {
        if (await invoke("vault_exists")) {
            (
                document.querySelector("input") as HTMLInputElement
            ).addEventListener("keypress", async (event) => {
                if (event.key !== "Enter") return;
                const invokeError: string = await invoke("load_vault", {
                    password: (
                        document.querySelector("input") as HTMLInputElement
                    ).value,
                });
                error = invokeError || "";
                window.location.href = "/keys";
            });
        } else {
            window.location.href = "/create-vault";
        }
    });
    async function unlockVault(event: Event) {
        event.preventDefault();
        const invokeError: string = await invoke("load_vault", {
            password: (document.querySelector("input") as HTMLInputElement)
                .value,
        });
        error = invokeError || "";
        window.location.href = "/keys";
    }

    // let isFirstOpenRes: Promise<boolean> = invoke("is_first_open");
</script>

<main class="container">
    <p>welcome back</p>
    <input type="password" placeholder="enter your vault password" />
    <button type="submit" onclick={unlockVault}>unlock vault</button>
    <p style:color="red">{error}</p>
</main>
