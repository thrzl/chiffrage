<script lang="ts">
    import { Channel } from "@tauri-apps/api/core";
    import { commands, type FileOperationProgress } from "$lib/bindings";
    import { open } from "@tauri-apps/plugin-dialog";
    import {formatBytes, getFileName} from "$lib/main"
    import * as Table from "$lib/components/ui/scroll-table/index";
    import * as Tabs from "$lib/components/ui/tabs/index";
    import * as Item from "$lib/components/ui/item/index";
    import Progress from "$lib/components/ui/progress/progress.svelte";
    import Label from "$lib/components/ui/label/label.svelte";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { TrashIcon, CircleQuestionMarkIcon } from "@lucide/svelte";
    import {andList} from "human-list";
    import * as Select from "$lib/components/ui/select/index.js";
    import {Button} from "$lib/components/ui/button/index";
    import { toast } from "svelte-sonner";
    import PasswordBox from "../../components/PasswordBox.svelte";
    import type { ZxcvbnResult } from "@zxcvbn-ts/core";
    import SlideAlert from "../../components/SlideAlert.svelte";
    import Switch from "$lib/components/ui/switch/switch.svelte";
    import * as Tooltip from "$lib/components/ui/tooltip/index";

    let progress: FileOperationProgress | null = $state(null);
    let chosenKeys: string[] = $state(new URLSearchParams(window.location.search).get("keys")?.split(",") ?? []);
    let files: string[] | null = $state(null);
    let armor = $state(false)
    let encryptMethod: "pass" | "key" = $state("key")

    let password = $state("");
    let strength = $state<ZxcvbnResult | null>(null);


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
            toast.error("no file selected");
            return;
        }
        if (encryptMethod === "key" && chosenKeys.length === 0) {
            toast.error("no key selected");
            return;
        }
        if (encryptMethod === "pass" && password.length === 0) {
            toast.error("no password set");
            return;
        }
        progress = null;
        const channel = new Channel<FileOperationProgress>();
        channel.onmessage = (msg) => {
            progress = msg;
        };
        let encryptRes = await commands.encryptFile(
            encryptMethod === "key" ? chosenKeys : password,
            channel,
            files,
            armor
        );
        if (encryptRes.status === "error") toast.error(encryptRes.error)
    }
    let keys = $state(await commands.fetchKeys());
    let privateKeys = keys.filter(key => key.key_type === "Private");
    let publicKeys = keys.filter(key => key.key_type === "Public");
    let keyMap = $derived(Object.fromEntries(keys.map(key => ([key.id, key]))));

  let alert: { title: string; description: string } | undefined = $derived.by(
      () => {
          if (encryptMethod === "key" && !(chosenKeys.length === 0 || chosenKeys.some(id => keyMap[id].key_type === "Private"))) {
             return {title: "consider adding a private key", description: "if you do not encrypt to one of your own keys, you will not be able to decrypt this file later."}
          } else if (encryptMethod === "pass" && strength && strength.guessesLog10 < 5) {
              let feedback = strength.feedback;
              return {
                  title: "weak password",
                  description:
                      `${feedback.warning ? feedback.warning.toLocaleLowerCase() + " " : ""}${feedback.suggestions[0].toLocaleLowerCase()}` ||
                      "this password is not very secure.",
              };
          }
      },
  );
</script>

<main class="container">
    <h1 class="text-2xl font-bold mb-2">encrypt</h1>

    <form onsubmit={chooseFile} class="w-4/5 mx-auto">
            <Tabs.Root bind:value={encryptMethod} onValueChange={(v) => v === "key" ? password = "" : chosenKeys = []}><Tabs.List class="w-full">
                <Tabs.Trigger value="key">keys</Tabs.Trigger>
                <Tabs.Trigger value="pass">passphrase</Tabs.Trigger>
            </Tabs.List>
        <div class="flex flex-row gap-2 justify-items-center justify-center mx-auto w-full">
            <Tabs.Content value="key" class="grow w-45 mb-2">
                <Select.Root type="multiple" name="target keys" bind:value={chosenKeys}>
                    <Select.Trigger class="grow w-full">
                        <p>{@html chosenKeys.length > 0 ? andList(chosenKeys.map(id => `<span class="font-bold">${keyMap[id].name}</span>`)) : "choose recipients..."}</p>
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
            </Tabs.Content>
            <Tabs.Content value="pass" class="grow w-45">
                <PasswordBox bind:password={password} bind:strength={strength} textAlign={"left"}/>
            </Tabs.Content>
                <Button onclick={chooseFile} variant={"secondary"}
                    >{files
                        ? `${files.length} files selected`
                        : "choose file(s)"}</Button
                >
        </div>
            </Tabs.Root>
            <Item.Root variant="outline" class="bg-secondary mb-2 p-4">
                <Item.Content class="text-left">
                    <Item.Title>
                        armor with ASCII?
                        <Tooltip.Provider delayDuration={200}>
                          <Tooltip.Root>
                            <Tooltip.Trigger
                              ><CircleQuestionMarkIcon class="h-4"/></Tooltip.Trigger
                            >
                            <Tooltip.Content class="bg-secondary text-secondary-foreground max-w-64 border-outline border shadow-lg" arrowClasses="h-0">
                              <p>use ASCII characters instead of binary. slightly increased compatibility at the cost of higher file size.</p>
                            </Tooltip.Content>
                          </Tooltip.Root>
                        </Tooltip.Provider>
                    </Item.Title>
                </Item.Content>
                <Item.Actions>
                    <Switch
                        bind:checked={armor}
                        style={armor ? "--primary: lightgreen" : ""}
                    />
                </Item.Actions>
            </Item.Root>
        <SlideAlert bind:alert />
        <Button
            onclick={encryptFile}
            disabled={(encryptMethod === "key" ? chosenKeys.length  : password.length) === 0 || !files || (progress && progress.read_bytes !== progress.total_bytes)}
            class="w-full rounded-b-none truncate">{#if progress && progress.read_bytes !== progress.total_bytes}<Spinner/> encrypting {progress.current_file}
                {:else}encrypt{/if}</Button
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
                </Table.Row>{/each}
            {/if}
        </Table.Body>
    </Table.Root>
        <!-- </ScrollArea> -->
    </div>
</main>

<style>
</style>
