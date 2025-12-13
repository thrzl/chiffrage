<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";
    import { open } from "@tauri-apps/plugin-dialog";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

    type Key = {
        id: string;
        name: string;
        key_type: "public" | "private";
        date_created: Date;
    };

    let name = $state("");
    let error = $state("");
    const tauriWindow = getCurrentWebviewWindow();

    function getName() {
        if (!name) {
            error = "no name set!";
            return;
        }
        if (!name.match(/^[a-zA-Z0-9]+$/)) {
            error = "name may only consist of alphanumeric characters";
            return;
        }
        return name;
    }
    async function generate_key(event: Event) {
        event.preventDefault();
        if (!getName()) return;
        error = "";
        // Learn more about Tauri commands at https://tauri.app/d,evelop/calling-rust/
        await invoke("generate_keypair", { name: name.trim() });
        emit("update-keys");
        await tauriWindow.close();
    }
    async function import_key(event: Event) {
        event.preventDefault();
        if (!getName()) return;
        error = "";
        let path = await open({ directory: false, multiple: false });
        if (!path) return;

        console.log(await invoke("import_key", { name: name.trim(), path }));
        console.log("imported key");
        emit("update-keys");
        await tauriWindow.close();
    }
    const keysFetch: Promise<Key[]> = invoke("fetch_keys");
    // console.log(`keys: ${await invoke("keys")}`);
</script>

<main class="container">
    <h1>add a key</h1>

    <form>
        <input bind:value={name} placeholder="key name" required />
        <button onclick={generate_key}>generate keypair</button>
        <p>or...</p>
        <button onclick={import_key}>import key</button>
    </form>
    <p style:color="red">{error}</p>
</main>
