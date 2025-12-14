<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Key } from "$lib/main";

    let error = $state("");
    let progress = $state(0);
    let key = $state("");
    let file: string = $state("choose file");

    async function chooseFile(event: Event) {
        event.preventDefault();
        file =
            (await open({
                multiple: false,
                directory: false,
            })) || "choose file";
    }
    async function encryptFile(event: Event) {
        event.preventDefault();
        if (file === "choose file") {
            error = "no file selected";
            return;
        }
        if (!key) {
            error = "no key selected";
            return;
        }
        error = "";
        progress = 0;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            progress = message;
        };
        error = await invoke("encrypt_file_cmd", {
            publicKeys: [key],
            reader: channel,
            file,
        });
    }
    let keysFetch: Promise<Key[]> = $state(invoke("fetch_keys"));
    // listen("update-keys", () => (keysFetch = invoke("fetch_keys")));
</script>

<main class="container">
    <h1>encrypt to key</h1>

    <form onsubmit={chooseFile}>
        <select bind:value={key}>
            {#await keysFetch}
                <option value="" disabled selected>loading keys</option>
            {:then keys}
                <option value="" disabled selected>choose a key</option>
                {#if keys}
                    {#each keys as key}
                        <option value={key.id}>{key.name}</option>
                    {/each}
                {:else}
                    <option value="no-key" disabled>no keys!</option>
                {/if}
            {/await}
        </select>
        <button onclick={chooseFile}
            >{file.split("/").slice(-1) || "choose file"}</button
        >
        <button
            onclick={encryptFile}
            style:width="75%"
            style:margin="2rem"
            style:margin-top="0.5rem">encrypt file</button
        >
    </form>
    <div
        style="background-color: green; height: 10px"
        style:width={`${progress * 100}%`}
    ></div>
    <p>{error}</p>
</main>
