<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Key, Progress } from "$lib/main";

    let error = $state("");
    let message = $state("");
    let progress: Progress | null = $state(null);
    let key = $state("");
    let files: string[] | null = $state(null);

    async function chooseFile(event: Event) {
        event.preventDefault();
        files = await open({
            multiple: true,
            directory: false,
        });
    }
    async function encryptFile(event: Event) {
        event.preventDefault();
        if (!files) {
            error = "no file selected";
            return;
        }
        if (!key) {
            error = "no key selected";
            return;
        }
        error = "";
        progress = null;
        const channel = new Channel<Progress>();
        channel.onmessage = (msg) => {
            progress = msg;
            if (progress.read_bytes !== progress.total_bytes) {
                return (message = `encrypting <br/>${progress.current_file}`);
            }
            message = "";
        };
        error = await invoke("encrypt_file_cmd", {
            publicKeys: [key],
            reader: channel,
            files,
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
            >{files
                ? `${files.length} files selected`
                : "choose file(s)"}</button
        >
        <button
            onclick={encryptFile}
            style:width="75%"
            style:margin="2rem"
            style:margin-top="0.5rem">encrypt</button
        >
    </form>
    <div
        style="background-color: green; height: 10px"
        style:width={progress
            ? `${(progress.read_bytes / progress.total_bytes) * 100}%`
            : "0"}
    ></div>
    <p bind:innerHTML={message} contenteditable></p>
    <p>{error}</p>
</main>
