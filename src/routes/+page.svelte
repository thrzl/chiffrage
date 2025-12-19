<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { openWindow, getFileName } from "$lib/main";
    import type { Key } from "$lib/main";
    import { Button, buttonVariants } from "$lib/components/ui/button/index";
    import * as Table from "$lib/components/ui/table/index";
    import * as Empty from "$lib/components/ui/empty/index";
    import * as Dialog from "$lib/components/ui/dialog/index";
    import * as Item from "$lib/components/ui/item/index";
    import { Label } from "$lib/components/ui/label/index";
    import { Input } from "$lib/components/ui/input/index";
    import Plus from "@lucide/svelte/icons/plus";
    import Lock from "@lucide/svelte/icons/lock";
    import Unlock from "@lucide/svelte/icons/lock-open";
    import KeyIcon from "@lucide/svelte/icons/key";
    import { open } from "@tauri-apps/plugin-dialog";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index";

    let name = $state("");
    let keyFile: string | null = $state(null);
    let greetMsg = $state("");
    let error = $state("");
    let keygenDialogOpen = $state(false);
    let keyImportDialogOpen = $state(false);

    function getName() {
        if (!name) {
            error = "no name set!";
            return;
        }
        return name;
    }
    if (!(await invoke("vault_exists"))) {
        window.location.href = "/create-vault";
    }
    async function generate_key() {
        if (!getName()) return;
        error = "";
        // Learn more about Tauri commands at https://tauri.app/d,evelop/calling-rust/
        if (!(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        await invoke("generate_keypair", { name: name.trim() });
        keys = await invoke("fetch_keys");
        keygenDialogOpen = false
    }
    async function chooseKeyFile() {
      keyFile = await open({
          directory: false,
          multiple: false,
          filters: [{ name: "age keyfiles", extensions: [".age", ".txt"] }],
      });
    }
    async function import_key() {
        if (!getName()) return error = "no name set";
        if (!keyFile) return error = "no file selected";
        error = "";

        if (!(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        console.log(await invoke("import_key", { name: name.trim(), path: keyFile }));
        console.log("imported key");
        keys = await invoke("fetch_keys");
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
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props })}
              <Button {...props}><Plus />new key</Button>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.Content class="w-56" align="start">
              <DropdownMenu.Item onSelect={() => keygenDialogOpen = true}>
                  generate key
                <!-- <DropdownMenu.Shortcut>⇧⌘P</DropdownMenu.Shortcut> -->
              </DropdownMenu.Item>
              <DropdownMenu.Item onSelect={() => keyImportDialogOpen = true}>
                  import key
              </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
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
                        <Table.Cell>{key.key_type.toLowerCase()}</Table.Cell>
                        <Table.Cell>{key.name}</Table.Cell>
                        <Table.Cell
                            >{new Date(
                                key.date_created.secs_since_epoch * 1000,
                            ).toLocaleDateString()}</Table.Cell
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

                    <Button onclick={() => keygenDialogOpen = true}>new keypair</Button>
                    <Button onclick={() => keyImportDialogOpen = true} variant="outline">import key</Button>
                </div>
            </Empty.Content>
        </Empty.Root>
    {/if}
</main>
<Dialog.Root bind:open={keygenDialogOpen}>
    <form>
        <!-- <Dialog.Trigger
            class={buttonVariants({ variant: "default" })}
            >new keypair</Dialog.Trigger
        > -->
        <Dialog.Content class="sm:max-w-[425px]">
            <Dialog.Header>
                <Dialog.Title
                    >generate new keypair</Dialog.Title
                >
                <Dialog.Description>
                    this will generate a public and private key. this
                    action requires authentication in order to encrypt
                    your private key.
                </Dialog.Description>
            </Dialog.Header>
            <div class="grid gap-4">
                <div class="grid gap-3">
                    <Label for="name-1">name</Label>
                    <Input
                        id="name-1"
                        name="name"
                        required
                        bind:value={name}
                    />
                </div>
            </div>
            <Dialog.Footer>
                <Dialog.Close
                    class={buttonVariants({
                        variant: "outline",
                    })}
                    onclick={() => keygenDialogOpen = false}>cancel</Dialog.Close
                >
                <Dialog.Close
                    class={buttonVariants({
                        variant: "default",
                    })}
                    onclick={generate_key}
                    disabled={name === ""}
                    >generate</Dialog.Close
                >
            </Dialog.Footer>
        </Dialog.Content>
    </form>
</Dialog.Root>

<Dialog.Root bind:open={keyImportDialogOpen}>
    <form>
        <Dialog.Content class="sm:max-w-[425px]">
            <Dialog.Header>
                <Dialog.Title
                    >import key</Dialog.Title
                >
                <Dialog.Description>
                    import a key from a file
                </Dialog.Description>
            </Dialog.Header>
            <div class="grid gap-4">
                <div class="grid gap-3">
                    <Label for="name-1">name</Label>
                    <Input
                        id="name-1"
                        name="name"
                        required
                        bind:value={name}
                    />
                    <Item.Root variant="outline" class="border-dashed">
                        <Item.Content>
                          <Item.Title>key file</Item.Title>
                          <Item.Description
                            >click here to import a file.</Item.Description
                          >
                        </Item.Content>
                        <Item.Actions>
                          <Button variant="outline" size="sm" onclick={chooseKeyFile}>{keyFile ? getFileName(keyFile) : "choose file"}</Button>
                        </Item.Actions>
                      </Item.Root>
                </div>
            </div>
            <Dialog.Footer>
                <Dialog.Close
                    class={buttonVariants({
                        variant: "outline",
                    })}
                    onclick={() => keyImportDialogOpen = false}>cancel</Dialog.Close
                >
                <Dialog.Close
                    class={buttonVariants({
                        variant: "default",
                    })}
                    onclick={import_key}
                    disabled={name === "" || !keyFile}
                    >import key</Dialog.Close
                >
            </Dialog.Footer>
        </Dialog.Content>
    </form>
</Dialog.Root>

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
