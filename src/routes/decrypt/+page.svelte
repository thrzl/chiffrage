<script lang="ts">
    import { invoke, Channel } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { toast } from "svelte-sonner";
    import type { Key, Progress as FileProgress } from "$lib/main";
    import * as Table from "$lib/components/ui/scroll-table";
    import * as Select from "$lib/components/ui/select/index";
    import * as Tabs from "$lib/components/ui/tabs/index"
    import Label from "$lib/components/ui/label/label.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { Progress } from "$lib/components/ui/progress";
    import { TrashIcon } from "@lucide/svelte";
    import { getFileName, formatBytes } from "$lib/main";
    import PasswordBox from "../../components/PasswordBox.svelte";

    let progress: FileProgress | null = $state(null);
    let password = $state("");
    let chosenKey = $state("");
    let files: string[] | null = $state(null);
    let decryptMethod: "Scrypt" | "X25519" = $state("X25519");
    const methodMap = {
      pass: "Scrypt",
      key: "X25519"
    }

    async function chooseFile(event: Event) {
        event.preventDefault();
        files = await open({
            multiple: true,
            directory: false,
        });
    }
    async function decryptFile(event: Event) {
        event.preventDefault();
        if (decryptMethod === "X25519" && !(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        progress = null;
        const channel = new Channel<FileProgress>();
        channel.onmessage = (msg) => {
            progress = msg;
        };
        invoke("decrypt_file", {
            privateKey: decryptMethod === "X25519" ? chosenKey : password,
            reader: channel,
            files,
            method: decryptMethod
        }).then(() => progress?.read_bytes === progress?.total_bytes)
        .catch(e => toast.error(e));
    }
    let keyFetch: Key[] = $state(await invoke("fetch_keys"));
    let keys = keyFetch.filter(key => key.key_type === "Private");
    let keyMap = $derived(Object.fromEntries(keys.map(key => ([key.id, key]))));
</script>

<main class="container ">
    <h1 class="text-2xl font-bold mb-2">decrypt</h1>

    <form onsubmit={chooseFile} class="w-4/5 mx-auto">
        <Tabs.Root bind:value={decryptMethod}><Tabs.List class="w-full">
            <Tabs.Trigger value="X25519">keys</Tabs.Trigger>
            <Tabs.Trigger value="Scrypt">passphrase</Tabs.Trigger>
        </Tabs.List>
        <div class="flex flex-row gap-2 justify-items-center justify-center mx-auto w-full">
            <Tabs.Content value="X25519" class="flex-grow w-[180px]">
        <Select.Root type="single" name="target key" bind:value={chosenKey}>
          <Select.Trigger class="w-full">
              <p>{#if chosenKey}<span class="font-bold">{keyMap[chosenKey].name}</span>{:else}choose recipient...{/if}</p>
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
            </Tabs.Content>
            <Tabs.Content value="Scrypt" class="flex-grow w-[180px]">
                <PasswordBox bind:password={password} showMeter={false} showGenerate={false}/>
            </Tabs.Content>
        <Button onclick={chooseFile} variant={"secondary"}
            >{files
                ? `${files.length} files selected`
                : "choose file(s)"}</Button
        >
        </div>
        </Tabs.Root>
        <Button
            onclick={decryptFile}
            disabled={(decryptMethod === "X25519" ? chosenKey.length  : password.length) === 0 || !files || (progress && progress.read_bytes !== progress.total_bytes)}
            class="w-full mt-2 rounded-b-none">{#if progress && progress.read_bytes !== progress.total_bytes}<Spinner/> decrypting {progress.current_file.replace(/\.age$/,"")}
                {:else}decrypt{/if}</Button
        >
        <Progress
            min={0}
            max={progress?.total_bytes}
            value={progress?.read_bytes}
            class="rounded-t-none"
            id="progress-bar"
            style={`--primary: ${!progress || progress.read_bytes === 0 ? "inherit" : "lightgreen"}`}
        />
        <Label for="progress-bar" class="mt-2 text-xs text-center mx-auto block">{formatBytes(progress?.read_bytes || 0)} / {formatBytes(progress?.total_bytes || 0)}</Label>
    </form>
    <div class="w-4/5 mx-auto mt-4">
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
                    <Table.Cell class="text-center"><Button variant={"secondary"} class="cursor-pointer" disabled><TrashIcon class="w-4"/></Button></Table.Cell>
                </Table.Row>
                {:else}
            {#each files as file}
                <Table.Row>
                    <Table.Cell class="truncate px-4">{getFileName(file)}</Table.Cell>
                    <Table.Cell class="text-center"><Button variant={"secondary"} class="cursor-pointer" onclick={() => files = files!.length > 1 ? files!.filter((f) => f !== file) : null}><TrashIcon class="w-4"/></Button></Table.Cell>
                </Table.Row>{/each}{/if}
        </Table.Body>
    </Table.Root>
    </div>
</main>
