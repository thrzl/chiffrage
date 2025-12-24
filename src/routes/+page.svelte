<script lang="ts">
    import { listen } from "@tauri-apps/api/event";
    import { commands, type KeyMetadata } from "$lib/bindings";
    import { Button } from "$lib/components/ui/button/index";
    import * as Table from "$lib/components/ui/scroll-table/index";
    import * as Empty from "$lib/components/ui/empty/index";
    import KeyIcon from "@lucide/svelte/icons/key-round";
    import KeyImportDialog from "../components/KeyImportDialog.svelte";
    import KeyGenDialog from "../components/KeyGenDialog.svelte";
    import KeyViewDialog from "../components/KeyViewDialog.svelte";
    import { RefreshCcwDotIcon, FolderKeyIcon } from "@lucide/svelte";

    let keygenDialogOpen = $state(false);
    let keyImportDialogOpen = $state(false);
    let selectedKey: KeyMetadata | undefined = $state(undefined);
    if (!(await commands.vaultExists())) {
        window.location.href = "/create-vault";
    }
    let keys = $state(await commands.fetchKeys());
    listen("update-keys", async () => {
        keys = await commands.fetchKeys();
        selectedKey = undefined;
    });
    console.log("hi");
</script>

<main class="container">
    <h1 class="text-2xl font-bold mb-2">your keys</h1>
    {#if keys.length > 0}
        <nav>
            <Button
                variant={"secondary"}
                onclick={() => (keygenDialogOpen = true)}
                ><RefreshCcwDotIcon /> generate key</Button
            >
            <Button
                variant={"secondary"}
                onclick={() => (keyImportDialogOpen = true)}
                ><FolderKeyIcon /> import key</Button
            >
        </nav>
        <Table.Root
            height={"16rem"}
            style="text-align: left; max-width: 100vw"
            containerClass="m-2"
        >
            <Table.Header>
                <Table.Row>
                    <Table.Head class="sticky top-0 bg-secondary"
                        >type</Table.Head
                    >
                    <Table.Head class="sticky top-0 bg-secondary"
                        >name</Table.Head
                    >
                    <Table.Head class="sticky top-0 bg-secondary"
                        >date created</Table.Head
                    >
                </Table.Row>
            </Table.Header>
            <Table.Body>
                {#each keys as key}
                    <Table.Row
                        class="cursor-pointer"
                        onclick={() => {
                            {
                                console.log(`selected key: ${key.id}`);
                                selectedKey = key;
                            }
                        }}
                    >
                        <Table.Cell>{key.key_type.toLowerCase()}</Table.Cell>
                        <Table.Cell>{key.name}</Table.Cell>
                        <Table.Cell
                            >{new Date(
                                // @ts-ignore 2339
                                key.date_created.secs_since_epoch * 1000,
                            ).toLocaleDateString()}</Table.Cell
                        >
                    </Table.Row>{/each}
            </Table.Body>
        </Table.Root>
    {:else}
        <Empty.Root class="p-0! max-h-48 m-8">
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
                    <Button onclick={() => (keygenDialogOpen = true)}
                        >new keypair</Button
                    >
                    <Button
                        onclick={() => (keyImportDialogOpen = true)}
                        variant="outline">import key</Button
                    >
                </div>
            </Empty.Content>
        </Empty.Root>
    {/if}
</main>

<KeyViewDialog bind:key={selectedKey} />
<KeyImportDialog bind:open={keyImportDialogOpen} />
<KeyGenDialog bind:open={keygenDialogOpen} />

<style>
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
