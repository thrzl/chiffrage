<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { openWindow } from "$lib/main";
    import type { PageProps } from "./$types";
    import type { Key } from "$lib/main";
    import { ask } from "@tauri-apps/plugin-dialog";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { emit } from "@tauri-apps/api/event";

    // Prints boolean to the console

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
        await invoke("delete_key", { name: slug });
        emit("update-keys");
        await webviewWindow.close();
    }
</script>

<main class="container">
    <h1>key info</h1>
    <h2>{key.name}</h2>
    <p>has private key? {isPrivateKey ? "yes" : "no"}</p>
    <nav>
        <button onclick={() => openWindow("produce-age", "encrypt")}
            >encrypt</button
        >
        {#if isPrivateKey}<button
                onclick={() => openWindow("consume-age", "decrypt")}
                >decrypt</button
            >{/if}
        <button onclick={deleteKey}>delete key</button>
        <button onclick={() => openWindow(`keys/${key.name}/export`, "export")}
            >export</button
        >
    </nav>
</main>
