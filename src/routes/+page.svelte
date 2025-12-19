<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { openWindow } from "$lib/main";
    import type { Key } from "$lib/main";
    import Button from "$lib/components/ui/button/button.svelte";
    import * as Table from "$lib/components/ui/table/index";
    import * as Empty from "$lib/components/ui/empty/index";
    import Plus from "@lucide/svelte/icons/plus";
    import Lock from "@lucide/svelte/icons/lock";
    import Unlock from "@lucide/svelte/icons/lock-open";
    import KeyIcon from "@lucide/svelte/icons/key";

    let name = $state("");
    let greetMsg = $state("");

    if (!(await invoke("vault_exists"))) {
        window.location.href = "/create-vault";
    }
    let keys: Key[] = $state(await invoke("fetch_keys"));
    listen("update-keys", async () => (keys = await invoke("fetch_keys")));
</script>

<main class="container">
    <h1 class="text-4xl font-bold mb-8">your keys</h1>
    <nav>
        <Button onclick={() => openWindow("encrypt", "encrypt")}
            ><Lock /> encrypt</Button
        >
        <Button onclick={() => openWindow("decrypt", "decrypt")}
            ><Unlock /> decrypt</Button
        >
        <Button onclick={() => openWindow("new-key", "new key")}
            ><Plus /> new key</Button
        >
    </nav>

    {#if keys.length > 0}
        <Table.Root style="text-align: left; margin: 2rem; max-width: 100vw">
            <Table.Header>
                <Table.Row>
                    <Table.Head>type</Table.Head>
                    <Table.Head>name</Table.Head>
                    <Table.Head>date created</Table.Head>
                </Table.Row>
            </Table.Header>
            <Table.Body>
                {#each keys as key}
                    <Table.Row
                        class="cursor-pointer"
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
                    </Table.Row>{/each}
            </Table.Body>
        </Table.Root>
    {:else}
        <Empty.Root>
            <Empty.Header>
                <Empty.Media variant="icon">
                    <KeyIcon />
                </Empty.Media>
                <Empty.Title>no keys yet</Empty.Title>
                <Empty.Description>
                    you haven't created or imported any keys yet.
                </Empty.Description>
            </Empty.Header>
            <Empty.Content>
                <div class="flex gap-2">
                    <Button>new keypair</Button>
                    <Button variant="outline">import key</Button>
                </div>
            </Empty.Content>
        </Empty.Root>
    {/if}
</main>

<style>
    tr[data-slot="table-row"] {
        cursor: pointer;
    }
    .container {
        margin: 3rem;
        /*padding-top: 10vh;*/
        display: flex;
        flex-direction: column;
        justify-content: center;
        text-align: center;
        max-width: 100vw;
    }
</style>
