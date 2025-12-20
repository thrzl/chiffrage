<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Key, Progress } from "$lib/main";
    import {getFileName} from "$lib/main"
    import * as Table from "$lib/components/ui/scroll-table/index";
    import * as Alert from "$lib/components/ui/alert/index";
    import Label from "$lib/components/ui/label/label.svelte";
    import {animate} from "motion/mini"
    import { TrashIcon, TriangleAlert } from "@lucide/svelte";
    import {andList} from "human-list";
    import * as Select from "$lib/components/ui/select/index.js";
    import Button from "$lib/components/ui/button/button.svelte";

    let error = $state("");
    let message = $state("");
    let progress: Progress | null = $state(null);
    let chosenKeys: string[] = $state(new URLSearchParams(window.location.search).get("keys")?.split(",") ?? []);
    let files: string[] | null = $state(null);


    let alertElement: HTMLDivElement | undefined = $state();

    async function chooseFile(event: Event) {
        event.preventDefault();
        let newFiles = await open({
            multiple: true,
            directory: false,
        });
        if (newFiles) files = [...new Set([...(files || []), ...newFiles])]
    }
    async function encryptFile(event: Event) {
        event.preventDefault();
        if (!files) {
            error = "no file selected";
            return;
        }
        if (chosenKeys.length === 0) {
            error = "no key selected";
            return;
        }
        error = "";
        progress = null;
        const channel = new Channel<Progress>();
        channel.onmessage = (msg) => {
            progress = msg;
            if (progress.read_bytes !== progress.total_bytes) {
                return (message = `encrypting <br/>${progress.current_file}`);
            }
            message = "";
        };
        error = await invoke("encrypt_file_cmd", {
            publicKeys: chosenKeys,
            reader: channel,
            files,
        });
    }
    let keys: Key[] = $state(await invoke("fetch_keys"));
    let privateKeys = keys.filter(key => key.key_type === "Private");
    let publicKeys = keys.filter(key => key.key_type === "Public");
    let keyMap = $derived(Object.fromEntries(keys.map(key => ([key.id, key]))));
    function updateAlert() {
      if (!alertElement) return
      let shouldAlert = !(chosenKeys.length === 0 || chosenKeys.some(id => keyMap[id].key_type === "Private"));
        animate(alertElement, {
          height: shouldAlert ? `${alertElement.scrollHeight}px` : 0
        }, { duration: 0.2, ease: "easeOut" }).then(() => alertElement!.style.marginTop = shouldAlert ? "0.5rem" : "0")
    }
    // listen("update-keys", () => (keysFetch = invoke("fetch_keys")));
</script>

<main class="container">
    <h1 class="text-2xl font-bold mb-2">encrypt to key</h1>

    <form onsubmit={chooseFile} class="w-3/4 mx-auto">
        <div class="flex flex-row gap-2 justify-items-center justify-center mx-auto">
        <Select.Root type="multiple" name="target keys" bind:value={chosenKeys} onValueChange={updateAlert}>
          <Select.Trigger class="w-[180px] flex-grow">
            <p>{@html chosenKeys.length > 0 ? andList(chosenKeys.map(id => `<span class="font-bold">${keyMap[id].name}</span>`)) : "choose keys..."}</p>
          </Select.Trigger>
          <Select.Content>
            {#if publicKeys.length > 0}
            <Select.Group>
              <Select.Label>public keys</Select.Label>
              {#each publicKeys as key}
                <Select.Item
                  value={key.id}
                  label={key.name}
                >
                  {key.name}
                </Select.Item>
              {/each}
            </Select.Group>
            {/if}
            {#if privateKeys.length > 0}
                <Select.Group>
              <Select.Label>private keys</Select.Label>
              {#each privateKeys as key}
                <Select.Item
                  value={key.id}
                  label={key.name}
                >
                  {key.name}
                </Select.Item>
              {/each}
            </Select.Group>
            {/if}
            {#if publicKeys.length === 0 && privateKeys.length === 0}
                <Select.Item
                  value={"#"}
                  label={"no key"}
                  disabled
                >
                  {"no keys"}
                </Select.Item>
            {/if}
          </Select.Content>
        </Select.Root>
        <Button onclick={chooseFile} variant={"secondary"}
            >{files
                ? `${files.length} files selected`
                : "choose file(s)"}</Button
        >
        </div>
        <div bind:this={alertElement} class="text-left overflow-hidden mt-0 h-0">
        <Alert.Root>
            <TriangleAlert />
            <Alert.Title>consider adding a private key</Alert.Title>
            <Alert.Description>without encrypting to one of your own keys, it will not be possible for you to decrypt it later</Alert.Description>
        </Alert.Root></div>
        <Button
            onclick={encryptFile}
            disabled={chosenKeys.length === 0 || !files}
            class="w-full mt-2">encrypt</Button
        >
    </form>
    <div class="w-3/4 mx-auto mt-4">
    <Label for="selected-files" class="mb-2">selected files</Label>
    <Table.Root height="12rem" id="selected-files" class="table-fixed">
        <Table.Header>
            <Table.Row>
                <Table.Head class="pl-4 sticky top-0 bg-secondary w-4/5">name</Table.Head>
                <Table.Head class="pl-4 sticky top-0 bg-secondary w-1/5">remove</Table.Head>
            </Table.Row>
        </Table.Header>
        <Table.Body>
            {#each files as file}
                <Table.Row>
                    <Table.Cell class="truncate">{getFileName(file)}</Table.Cell>
                    <Table.Cell><Button variant={"secondary"} class="cursor-pointer" onclick={() => files = files!.length > 1 ? files!.filter((f) => f !== file) : null}><TrashIcon class="w-4"/></Button></Table.Cell>
                </Table.Row>{/each}
        </Table.Body>
    </Table.Root>
        <!-- </ScrollArea> -->
    </div>
    <div
        style="background-color: green; height: 10px"
        style:width={progress
            ? `${(progress.read_bytes / progress.total_bytes) * 100}%`
            : "0"}
    ></div>
    <p>{@html message}</p>
    <p>{error}</p>
</main>

<style>
</style>
