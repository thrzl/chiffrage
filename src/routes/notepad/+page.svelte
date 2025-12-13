<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";

    type Key = {
        id: string;
        name: string;
        key_type: "public" | "private";
        date_created: Date;
    };

    let name = $state("");
    let key: { public_key: string; private_key: string } | undefined = $state();

    async function encrypt(event: Event) {
        key = (await invoke("encrypt_text", { id: name })) as {
            public_key: string;
            private_key: string;
        };
        console.log("generated keys");
        emit("update-keys");
    }
    const keysFetch: Promise<Key[]> = invoke("fetch_keys");
    // console.log(`keys: ${await invoke("keys")}`);
</script>

<main class="container">
    <h1>notepad</h1>

    <textarea bind:value={name} placeholder="key name"></textarea>
    <button onclick={encrypt}>generate keypair</button>
</main>
