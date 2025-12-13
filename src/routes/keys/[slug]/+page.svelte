<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { openWindow } from "$lib/main";
    import type { PageProps } from "./$types";
    import type { Key } from "$lib/main";

    const { data }: PageProps = $props();
    const slug = data.slug;

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

    let keyMatches = filter_keys(keysFetch)[0];
    console.log(keyMatches);
</script>

<main class="container">
    <h1>key info</h1>
    <h2>{keyMatches.name}</h2>
    <p>has private key? {keyMatches.key_type === "Private" ? "yes" : "no"}</p>
    <nav>
        <button onclick={() => openWindow("produce-age", "encrypt")}
            >encrypt</button
        >
        {#if keyMatches.key_type === "Private"}<button
                onclick={() => openWindow("consume-age", "decrypt")}
                >decrypt</button
            >{/if}
        <button onclick={() => openWindow("new-key", "new key")}>new key</button
        >
        <button
            onclick={() =>
                openWindow(`keys/${keyMatches.name}/export`, "export")}
            >export</button
        >
    </nav>
</main>
