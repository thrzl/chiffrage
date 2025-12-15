<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { zxcvbn, zxcvbnOptions } from "@zxcvbn-ts/core";
    import * as zxcvbnCommonPackage from "@zxcvbn-ts/language-common";
    import * as zxcvbnEnPackage from "@zxcvbn-ts/language-en";

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
            window.location.href = "/keys";
        }
    }

    // let isFirstOpenRes: Promise<boolean> = invoke("is_first_open");
</script>

<main class="container">
    <p>welcome, let's make a vault to store your keys!</p>
    <h1>choose a password</h1>
    <input bind:value={password} type="password" />
    <div
        style={`background-color: green; width: ${Math.min(zxcvbn(password).crackTimesSeconds.offlineSlowHashing1e4PerSecond / 31_536_000, 1) * 100}%; height: 10px`}
    ></div>
    {#if password}<p>
            will take {zxcvbn(password).crackTimesDisplay
                .offlineSlowHashing1e4PerSecond} to crack
        </p>{/if}
    <button onclick={createVault} disabled={password === ""}>create</button>
</main>
