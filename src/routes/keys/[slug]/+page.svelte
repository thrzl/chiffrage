<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { openWindow } from "$lib/main";
    import type { PageProps } from "./$types";
    import { save, open } from "@tauri-apps/plugin-dialog";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import type { Key } from "$lib/main";
    import { ask } from "@tauri-apps/plugin-dialog";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { emit } from "@tauri-apps/api/event";
    import { Channel } from "@tauri-apps/api/core";

    const { data }: PageProps = $props();
    const slug = data.slug;

    let name = $state("");
    let key: Key = await invoke("fetch_key", { name: slug });
    let isPrivateKey = key.key_type === "Private";
    const webviewWindow = getCurrentWebviewWindow();

    async function deleteKey(e: Event) {
        e.preventDefault();

        const answer = await ask(
            "this action cannot be reverted. are you sure?",
            {
                title: "delete key",
                kind: "warning",
            },
        );

        if (!answer) return;
        await invoke("delete_key", { id: slug });
        emit("update-keys");
        await webviewWindow.close();
    }

    async function encrypt(e: Event) {
        e.preventDefault();

        const file = await open({ directory: false, multiple: false });
        if (!file) return;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            // progress = message;
        };
        await invoke("encrypt_file_cmd", {
            publicKeys: [key.id],
            reader: channel,
            file,
        });
    }
    async function decrypt(event: Event) {
        event.preventDefault();
        // progress = 0;
        const file = await open({ directory: false, multiple: false });
        if (!file) return;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            // progress = message;
        };
        await invoke("decrypt_file_cmd", {
            privateKey: key.id,
            reader: channel,
            file,
        });
    }

    async function exportKey(keyType: "public" | "private") {
        const destination = await save({
            filters: [{ name: "age key file", extensions: ["age"] }],
        });
        if (!destination) {
            return;
        }
        await invoke("export_key", {
            key: slug,
            path: destination,
            keyType,
        });
        revealItemInDir(destination);
    }
</script>

<main class="container">
    <h1>key info</h1>
    <h2>{key.name}</h2>
    <p>has private key? {isPrivateKey ? "yes" : "no"}</p>
    <nav>
        <button onclick={encrypt}>encrypt</button>
        {#if isPrivateKey}<button onclick={decrypt}>decrypt</button>{/if}
        <br />
        <h2>manage key</h2>
        <button onclick={deleteKey}>delete key</button>
        <button onclick={() => exportKey("public")}>export public</button>
        {#if isPrivateKey}<button onclick={() => exportKey("private")}
                >export private</button
            >{/if}
    </nav>
</main>
