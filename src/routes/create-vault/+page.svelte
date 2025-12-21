<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { ZxcvbnResult } from "@zxcvbn-ts/core";
    import Button from "$lib/components/ui/button/button.svelte";
    import * as Alert from "$lib/components/ui/alert/index";
    import PasswordBox from "../../components/PasswordBox.svelte";
    import { TriangleAlertIcon, CircleCheckIcon } from "@lucide/svelte";
    import { animate } from "motion/mini";

    let password: string = $state("");

    let error = $state("");

    async function createVault(event: Event) {
        event.preventDefault();
        if (!password) {
            error = "password input field could not be found";
            return;
        }
        error = await invoke("create_vault", { password: password });
        if (!error) {
            await invoke("load_vault", { password: password });
            password =
                "don't read the password please that would not be nice and i really don't think you should do that";
            window.location.href = "/";
        }
    }

    let alertElement = $state<HTMLDivElement>();
    let strength = $state<ZxcvbnResult>();
    let alert: { title: string; description: string } | undefined = $derived.by(
        () => {
            if (!alertElement) return;
            if (strength && strength.guessesLog10 < 5) {
                let feedback = strength.feedback;
                return {
                    title: "weak password",
                    description:
                        `${feedback.warning ? feedback.warning.toLocaleLowerCase() + " " : ""}${feedback.suggestions[0].toLocaleLowerCase()}` ||
                        "this password is not very secure.",
                };
            }
        },
    );
    $effect(() => {
        if (!alertElement) return;
        // new Promise((resolve) => setTimeout(resolve, 10)); // small sleep just to allow the dom to update
        animate(
            alertElement,
            {
                height: `${alert ? alertElement.scrollHeight : "0"}px`,
            },
            { duration: 0.2, ease: "easeOut" },
        ).then(() => {
            if (!alertElement) return;
            let margin = alert ? "0.5rem" : "0rem";
            alertElement.style.marginBottom = margin;
        });
    });
</script>

<main class="container">
    <p>welcome, let's make a vault to store your keys!</p>
    <h1 class="text-2xl font-bold mb-2 tracking-tight">choose a password</h1>
    <PasswordBox bind:password bind:strength />
    <div bind:this={alertElement} class="text-left overflow-hidden mb-0 h-0">
        <Alert.Root>
            <TriangleAlertIcon />
            <Alert.Title>{alert?.title}</Alert.Title>
            <Alert.Description>{alert?.description}</Alert.Description>
        </Alert.Root>
    </div>
    <Button onclick={createVault} disabled={password === ""}>create</Button>
</main>

<style>
    .container {
        margin: 3rem;
        /*padding-top: 10vh;*/
        display: flex;
        flex-direction: column;
        justify-content: center;
        text-align: center;
    }
</style>
