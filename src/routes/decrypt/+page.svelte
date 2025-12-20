<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Key, Progress } from "$lib/main";
    import * as Table from "$lib/components/ui/scroll-table";
    import * as Select from "$lib/components/ui/select/index";
    import Label from "$lib/components/ui/label/label.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import { TrashIcon } from "@lucide/svelte";
    import { getFileName } from "$lib/main";

    let progress: Progress | null = $state(null);
    let message = $state("");
    let chosenKey = $state("");
    let error = $state("");
    let files: string[] | null = $state(null);

    async function chooseFile(event: Event) {
        event.preventDefault();
        files = await open({
            multiple: true,
            directory: false,
        });
    }
    async function decryptFile(event: Event) {
        event.preventDefault();
        if (!(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        progress = null;
        const channel = new Channel<Progress>();
        channel.onmessage = (msg) => {
            progress = msg;
            if (progress.read_bytes !== progress.total_bytes) {
                return (message = `decrypting <br/>${progress.current_file}`);
            }
            message = "";
        };
        invoke("decrypt_file_cmd", {
            privateKey: chosenKey,
            reader: channel,
            files,
        }).then()
        .catch(e => error = e);
    }
    let keyFetch: Key[] = $state(await invoke("fetch_keys"));
    let keys = keyFetch.filter(key => key.key_type === "Private");
    let keyMap = $derived(Object.fromEntries(keys.map(key => ([key.id, key]))));
</script>

<main class="container ">
    <h1 class="text-2xl font-bold mb-2">decrypt</h1>

    <form onsubmit={chooseFile} class="w-3/4 mx-auto">
        <div class="flex flex-row gap-2 justify-items-center justify-center mx-auto">
        <Select.Root type="single" name="target key" bind:value={chosenKey}>
          <Select.Trigger class="w-[180px] flex-grow">
              <p>{#if chosenKey}<span class="font-bold">{keyMap[chosenKey].name}</span>{:else}choose key...{/if}</p>
          </Select.Trigger>
          <Select.Content>
              <Select.Group>
                <Select.Label>private keys</Select.Label>
            {#if keys.length > 0}
              {#each keys as key}
                <Select.Item
                  value={key.id}
                  label={key.name}
                >
                  {key.name}
                </Select.Item>
              {/each}
            {/if}
            {#if keys.length === 0}
                <Select.Item
                  value={"#"}
                  label={"no key"}
                  disabled
                >
                  {"no private keys"}
                </Select.Item>
            {/if}
              </Select.Group>
          </Select.Content>
        </Select.Root>
        <Button onclick={chooseFile} variant={"secondary"}
            >{files
                ? `${files.length} files selected`
                : "choose file(s)"}</Button
        >
        </div>
        <Button
            onclick={decryptFile}
            disabled={chosenKey.length === 0 || !files}
            class="w-full mt-2">decrypt</Button
        >
    </form>
    <div class="w-3/4 mx-auto mt-4">
    <Label for="selected-files" class="mb-2">selected files</Label>
    <Table.Root height="8rem" id="selected-files" class="table-fixed text-left" containerClass="border-2 border-solid rounded-sm">
        <Table.Header>
            <Table.Row>
                <Table.Head class="pl-4 sticky top-0 bg-secondary w-4/5">name</Table.Head>
                <Table.Head class="pl-4 sticky top-0 bg-secondary w-1/5">remove</Table.Head>
            </Table.Row>
        </Table.Header>
        <Table.Body>
            {#if !files || files.length === 0}
                <Table.Row class="pointer-events-none">
                    <Table.Cell class="truncate px-4 opacity-60">no files selected</Table.Cell>
                    <Table.Cell class="text-center"><Button variant={"secondary"} class="cursor-pointer" disabled onclick={() => files = files!.length > 1 ? files!.filter((f) => f !== file) : null}><TrashIcon class="w-4"/></Button></Table.Cell>
                </Table.Row>
                {:else}
            {#each files as file}
                <Table.Row>
                    <Table.Cell class="truncate px-4">{getFileName(file)}</Table.Cell>
                    <Table.Cell class="text-center"><Button variant={"secondary"} class="cursor-pointer" onclick={() => files = files!.length > 1 ? files!.filter((f) => f !== file) : null}><TrashIcon class="w-4"/></Button></Table.Cell>
                </Table.Row>{/each}{/if}
        </Table.Body>
    </Table.Root>
        <!-- </ScrollArea> -->
    <div
        style="background-color: green; height: 10px"
        style:width={progress
            ? `${(progress.read_bytes / progress.total_bytes) * 100}%`
            : "0"}
    ></div>
    <p>{@html message}</p>
    <p>{error}</p>
    </div>
</main>
