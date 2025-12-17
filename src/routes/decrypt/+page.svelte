<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Key } from "$lib/main";

    let name = $state("");
    let progress = $state(0);
    let key = $state("");
    let greetMsg = $state("");
    let file: string = $state("choose file");

    async function chooseFile(event: Event) {
        event.preventDefault();
        file =
            (await open({
                multiple: false,
                directory: false,
            })) || "file load failed";
    }
    async function decryptFile(event: Event) {
        if (!(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        event.preventDefault();
        progress = 0;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            progress = message;
        };
        greetMsg = await invoke("decrypt_file_cmd", {
            privateKey: key,
            reader: channel,
            file,
        });
    }
    let keysFetch: Promise<Key[]> = $state(invoke("fetch_keys"));
    // listen("update-keys", () => (keysFetch = invoke("fetch_keys")));
</script>

<main class="container">
    <h1>decrypt</h1>

    <form onsubmit={chooseFile}>
        <select bind:value={key}>
            {#await keysFetch}
                <option value="" disabled selected>loading keys</option>
            {:then keys}
                <option value="" disabled selected>choose a key</option>
                {#if keys}
                    {#each keys.filter((key) => key.key_type === "Private") as key}
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
            onclick={decryptFile}
            style:width="75%"
            style:margin="2rem"
            style:margin-top="0.5rem">decrypt file</button
        >
    </form>
    <div
        style="background-color: green; height: 10px"
        style:width={`${progress * 100}%`}
    ></div>
    <p>{greetMsg}</p>
</main>
