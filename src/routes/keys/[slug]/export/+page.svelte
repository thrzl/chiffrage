<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { save } from "@tauri-apps/plugin-dialog";
    import type { PageProps } from "./$types";

    const { data }: PageProps = $props();
    const slug = data.slug;
    type Key = {
        id: string;
        name: string;
        key_type: "public" | "private";
        date_created: { secs_since_epoch: number };
    };

    let name = $state("");
    let greetMsg = $state("");

    async function greet(event: Event) {
        event.preventDefault();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        console.log(`keys: ${await invoke("fetch_keys")}`);
        greetMsg = await invoke("greet", { name });
    }

    function filter_keys(keys: Key[]) {
        return keys.filter((key) => {
            return (
                key.name.split(":").slice(-1)[0] ===
                slug.split(":").slice(-1)[0]
            );
        });
    }
    let keysFetch: Key[] = $state(await invoke("fetch_keys"));
    listen("update-keys", async () => (keysFetch = await invoke("fetch_keys")));

    let keyMatches = filter_keys(keysFetch);

    let privateKey = keyMatches.find((key) => key.name.startsWith("priv:"));
    let publicKey = keyMatches.find((key) => key.name.startsWith("pub:"))!;
    let general: Key = {
        id: publicKey.name.split(":").slice(-1)[0],
        name: publicKey.name.split(":").slice(-1)[0],
        date_created: publicKey.date_created,
        key_type: "public",
    };

    async function exportKey(keyType: "pub" | "priv") {
        const destination = await save({
            filters: [{ name: "age key file", extensions: ["age"] }],
        });
        if (!destination) {
            return;
        }
        await invoke("export_key", {
            key: `${keyType}:${general.name}`,
            path: destination,
        });
    }
</script>

<main class="container">
    <h1>key info</h1>
    <h2>{general.name}</h2>
    <p>has private key? {privateKey ? "yes" : "no"}</p>
    <nav>
        <button onclick={() => exportKey("pub")}>export public</button>
        {#if privateKey}<button onclick={() => exportKey("priv")}
                >export private</button
            >{/if}
    </nav>
</main>
