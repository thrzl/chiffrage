<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { save } from "@tauri-apps/plugin-dialog";
    import type { PageProps } from "./$types";
    import type { Key } from "$lib/main";

    const { data }: PageProps = $props();
    const slug = data.slug;

    let name = $state("");
    let greetMsg = $state("");

    let key: Key = await invoke("fetch_key", { name: slug });
    let privateKey = key.key_type === "Private";

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
    }
</script>

<main class="container">
    <h1>key info</h1>
    <h2>{key.name}</h2>
    <p>has private key? {privateKey ? "yes" : "no"}</p>
    <nav>
        <button onclick={() => exportKey("public")}>export public</button>
        {#if privateKey}<button onclick={() => exportKey("private")}
                >export private</button
            >{/if}
    </nav>
</main>
