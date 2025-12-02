<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    onMount(async () => {
        if (await invoke("vault_exists")) {
            (
                document.querySelector("input") as HTMLInputElement
            ).addEventListener("keypress", async (event) => {
                const error = await invoke("load_vault", {
                    password: (
                        document.querySelector("input") as HTMLInputElement
                    ).value,
                });
                if (error) return console.log(error);
                window.location.href = "/home";
            });
        } else {
            window.location.href = "/create-vault";
        }
    });

    type Key = {
        id: string;
        name: string;
        key_type: "public" | "private";
        date_created: Date;
    };

    let passwordInput: HTMLInputElement | null = null;

    let error = $state("");

    async function unlockVault(event: Event) {
        event.preventDefault();
        if (!passwordInput) {
            error = "password input field could not be found";
            return;
        }
        error = await invoke("unlock_vault", { password: passwordInput.value });
        if (!error) {
            passwordInput.value = "";
            window.location.href = "/home";
        }
    }

    // let isFirstOpenRes: Promise<boolean> = invoke("is_first_open");
</script>

<main class="container">
    <p>welcome back :P</p>
    <input type="password" placeholder="enter your vault password" />
</main>
