<script lang="ts">
    import type { ZxcvbnResult } from "@zxcvbn-ts/core";
    import Button from "$lib/components/ui/button/button.svelte";
    import PasswordBox from "../../components/PasswordBox.svelte";
    import SlideAlert from "../../components/SlideAlert.svelte";
    import { commands } from "$lib/bindings";

    let password: string = $state("");

    async function createVault(event: Event) {
        event.preventDefault();
        if (!password) {
            alert = {
                title: "password input field could not be found",
                description:
                    "this is an internal error. try restarting the app.",
            };
            return;
        }
        let result = await commands.createVault(password);
        if (result.status === "error") {
            alert = {
                title: "unable to create vault",
                description: result.error,
            };
            return;
        }
        await commands.loadVault();
        password =
            "don't read the password please that would not be nice and i really don't think you should do that";
        window.location.href = "/";
    }

    let strength = $state<ZxcvbnResult>();
    let alert: { title: string; description: string } | undefined = $derived.by(
        () => {
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
</script>

<main class="container px-24">
    <p>welcome, let's make a vault to store your keys!</p>
    <h1 class="text-2xl font-bold mb-2 tracking-tight">choose a password</h1>
    <PasswordBox bind:password bind:strength />
    <SlideAlert bind:alert />
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
