<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { zxcvbn, zxcvbnOptions } from "@zxcvbn-ts/core";
    import * as zxcvbnCommonPackage from "@zxcvbn-ts/language-common";
    import * as zxcvbnEnPackage from "@zxcvbn-ts/language-en";
    import { Input } from "$lib/components/ui/input";
    import Button from "$lib/components/ui/button/button.svelte";
    import Progress from "$lib/components/ui/progress/progress.svelte";

    const options = {
        translations: zxcvbnEnPackage.translations,
        graphs: zxcvbnCommonPackage.adjacencyGraphs,
        dictionary: {
            ...zxcvbnCommonPackage.dictionary,
            ...zxcvbnEnPackage.dictionary,
        },
    };

    zxcvbnOptions.setOptions(options);

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

    // let isFirstOpenRes: Promise<boolean> = invoke("is_first_open");
    let passwordPercentage = $derived(
        password
            ? zxcvbn(password).crackTimesSeconds
                  .offlineSlowHashing1e4PerSecond / 31_536_000
            : 0,
    );

    let progressColor = $derived(
        passwordPercentage === 0
            ? "white"
            : passwordPercentage > 0.8
              ? "lightgreen"
              : passwordPercentage > 0.5
                ? "yellow"
                : "red",
    );
</script>

<main class="container">
    <p>welcome, let's make a vault to store your keys!</p>
    <h1 class="text-2xl font-bold mb-2 tracking-tight">choose a password</h1>
    <Input
        bind:value={password}
        type="password"
        style="border-bottom-left-radius: 0px; border-bottom-right-radius: 0"
    />
    <Progress
        min={0}
        max={1}
        value={passwordPercentage}
        style={`--primary: ${progressColor}; border-top-left-radius: 0px; border-top-right-radius: 0`}
    />
    <!-- <div
        style={`background-color: green; width: ${Math.min(zxcvbn(password).crackTimesSeconds.offlineSlowHashing1e4PerSecond / 31_536_000, 1) * 100}%; height: 10px`}
    ></div> -->
    <p class="my-2">
        {#if password}
            will take <span class="font-bold"
                >{zxcvbn(password).crackTimesDisplay
                    .offlineSlowHashing1e4PerSecond}</span
            > to crack
        {:else}enter a password
        {/if}
    </p>
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
