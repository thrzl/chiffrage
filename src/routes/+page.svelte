<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { openWindow } from "$lib/main";
    import type { Key } from "$lib/main";

    let name = $state("");
    let greetMsg = $state("");

    if (!(await invoke("vault_exists"))) {
        window.location.href = "/create-vault";
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

    <table style="text-align: left; margin: 2rem">
        <thead>
            <tr>
                <th>type</th>
                <th>name</th>
                <th>date created</th>
            </tr>
        </thead>
        <tbody>
            {#await keysFetch then keys}
                {#if keys.length > 0}
                    {#each keys as key}
                        <tr
                            onclick={() => {
                                openWindow(`/keys/${key.id}`, "key details");
                            }}
                        >
                            <td>{key.key_type.toLowerCase()}</td>
                            <td>{key.name}</td>
                            <td
                                >{new Date(
                                    key.date_created.secs_since_epoch * 1000,
                                ).toLocaleDateString()}</td
                            >
                        </tr>{/each}
                {:else}
                    <tr
                        onclick={() => {
                            openWindow(`/new-key`, "create your first key");
                        }}
                    >
                        <td>n/a</td>
                        <td>click here to add/create a new key</td>
                        <td>n/a</td>
                    </tr>
                {/if}
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
