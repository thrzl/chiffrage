<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { openWindow } from "$lib/main";
    import type { Key } from "$lib/main";

    let name = $state("");
    let greetMsg = $state("");

    async function greet(event: Event) {
        event.preventDefault();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        console.log(`keys: ${await invoke("fetch_keys")}`);
        greetMsg = await invoke("greet", { name });
    }

    let keysFetch: Promise<Key[]> = $state(invoke("fetch_keys"));
    listen("update-keys", () => (keysFetch = invoke("fetch_keys")));
</script>

<main class="container">
    <h1>chiffrage</h1>
    <nav>
        <button onclick={() => openWindow("encrypt", "encrypt")}>encrypt</button
        >
        <button onclick={() => openWindow("decrypt", "decrypt")}>decrypt</button
        >
        <button onclick={() => openWindow("new-key", "new key")}>new key</button
        >
    </nav>

    <div class="row">
        <a href="https://vite.dev" target="_blank">
            <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
            <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
        </a>
        <a href="https://svelte.dev" target="_blank">
            <img
                src="/svelte.svg"
                class="logo svelte-kit"
                alt="SvelteKit Logo"
            />
        </a>
    </div>
    <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

    <table style="text-align: left; margin: 0 2rem">
        <thead>
            <tr>
                <th>id</th>
                <th>private?</th>
            </tr>
        </thead>
        <tbody>
            {#await keysFetch then keys}
                {#each keys as key}
                    <tr
                        onclick={() => {
                            openWindow(`/keys/${key.name}`, "key details");
                        }}
                    >
                        <td>{key.name}</td>
                        <td>{key.key_type === "Private" ? "yes" : "no"}</td>
                    </tr>{/each}
            {/await}
        </tbody>
    </table>
</main>

<style>
    thead > tr {
        background-color: #222222;
        cursor: default !important;
    }
    tr {
        margin: 1rem;
        background-color: #444444;
    }
    tr:hover {
        cursor: pointer;
    }
    th,
    td {
        padding: 0.25rem 0.5rem;
    }
</style>
