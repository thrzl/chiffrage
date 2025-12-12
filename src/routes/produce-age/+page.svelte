<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";

    type Key = {
        id: string;
        name: string;
        key_type: "public" | "private";
        date_created: Date;
    };

    let name = $state("");
    let progress = $state(0);
    let key = $state("");
    let greetMsg = $state("");

    async function greet(event: Event) {
        event.preventDefault();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        progress = 0;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            progress = message;
        };
        greetMsg = await invoke("encrypt_file_cmd", {
            publicKeys: [key],
            reader: channel,
        });
    }
    let keysFetch: Promise<Key[]> = $state(invoke("fetch_keys"));
    // listen("update-keys", () => (keysFetch = invoke("fetch_keys")));
</script>

<main class="container">
    <h1>encrypt + sign</h1>

    <form class="row" onsubmit={greet}>
        <select bind:value={key}>
            {#await keysFetch}
                <option value="no-key" disabled>loading keys</option>
            {:then keys}
                {#if keys}
                    {#each keys.filter((key) => key.name.split(":", 1)[0] === "pub") as key}
                        <option value={key.name}>{key.name}</option>
                    {/each}
                {:else}
                    <option value="no-key" disabled>no keys!</option>
                {/if}
            {/await}
        </select>
        <button type="submit" onclick={greet}>choose file</button>
    </form>
    <div
        style="background-color: green; height: 10px"
        style:width={`${progress * 100}%`}
    ></div>
    <p>{greetMsg}</p>
</main>

<style>
    nav {
        flex-flow: row wrap;
    }
    .logo.vite:hover {
        filter: drop-shadow(0 0 2em #747bff);
    }

    .logo.svelte-kit:hover {
        filter: drop-shadow(0 0 2em #ff3e00);
    }

    :root {
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 24px;
        font-weight: 400;

        color: #0f0f0f;
        background-color: #f6f6f6;

        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        -webkit-text-size-adjust: 100%;
    }

    .container {
        margin: 0;
        padding-top: 10vh;
        display: flex;
        flex-direction: column;
        justify-content: center;
        text-align: center;
    }

    .logo {
        height: 6em;
        padding: 1.5em;
        will-change: filter;
        transition: 0.75s;
    }

    .logo.tauri:hover {
        filter: drop-shadow(0 0 2em #24c8db);
    }

    .row {
        display: flex;
        justify-content: center;
    }

    a {
        font-weight: 500;
        color: #646cff;
        text-decoration: inherit;
    }

    a:hover {
        color: #535bf2;
    }

    h1 {
        text-align: center;
    }

    input,
    button {
        border-radius: 8px;
        border: 1px solid transparent;
        padding: 0.6em 1.2em;
        font-size: 1em;
        font-weight: 500;
        font-family: inherit;
        color: #0f0f0f;
        background-color: #ffffff;
        transition: border-color 0.25s;
        box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    }

    button {
        cursor: pointer;
    }

    button:hover {
        border-color: #396cd8;
    }
    button:active {
        border-color: #396cd8;
        background-color: #e8e8e8;
    }

    input,
    button {
        outline: none;
    }

    #greet-input {
        margin-right: 5px;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            color: #f6f6f6;
            background-color: #2f2f2f;
        }

        a:hover {
            color: #24c8db;
        }

        input,
        button {
            color: #ffffff;
            background-color: #0f0f0f98;
        }
        button:active {
            background-color: #0f0f0f69;
        }
    }
</style>
