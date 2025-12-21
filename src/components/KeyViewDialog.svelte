<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index";
    import Confirm from "./Confirm.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import {
        CopyIcon,
        LockIcon,
        LockOpenIcon,
        TrashIcon,
        FolderUpIcon,
        SquareAsteriskIcon,
    } from "@lucide/svelte";
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import { toast } from "svelte-sonner";
    import * as InputGroup from "$lib/components/ui/input-group/index";
    import { invoke } from "@tauri-apps/api/core";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { ask, open, save } from "@tauri-apps/plugin-dialog";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import { Channel } from "@tauri-apps/api/core";
    import type { Key } from "$lib/main";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import Label from "$lib/components/ui/label/label.svelte";
    let webviewWindow = getCurrentWebviewWindow();
    let { key = $bindable() }: { key: Key | undefined } = $props();
    let hasKey = $derived(key !== undefined);
    let confirming = $state(false);
    let exportingKey = $state(false);
    let isPrivateKey = $derived(hasKey && key?.key_type === "Private");
    console.log(`key: ${key?.id}`);

    async function deleteKey(e: Event) {
        e.preventDefault();

        const answer = await ask(
            "this action cannot be reverted. are you sure?",
            {
                title: "delete key",
                kind: "warning",
            },
        );

        if (!answer) return;
        if (!(await invoke("authenticate"))) {
            return toast.error("authentication failed");
        }
        await invoke("delete_key", { id: key?.id });
        webviewWindow.emit("update-keys");
    }

    async function encrypt(e: Event) {
        e.preventDefault();

        const file = await open({ directory: false, multiple: false });
        if (!file) return;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            // progress = message;
        };
        await invoke("encrypt_file_cmd", {
            publicKeys: [key?.id],
            reader: channel,
            file,
        });
    }
    async function decrypt(event: Event) {
        event.preventDefault();
        // progress = 0;
        const file = await open({ directory: false, multiple: false });
        if (!file) return;
        const channel = new Channel<number>();
        channel.onmessage = (message) => {
            // progress = message;
        };
        await invoke("decrypt_file_cmd", {
            privateKey: key?.id,
            reader: channel,
            file,
        });
    }

    async function exportKey(keyType: "public" | "private") {
        if (keyType === "private") {
            let authComplete = await invoke("authenticate");
            if (!authComplete) {
                return toast.error("authentication failed");
            }
            exportingKey = true;
        }

        const destination = await save({
            filters: [{ name: "age key file", extensions: ["age"] }],
        });
        if (!destination) {
            exportingKey = false;
            return;
        }
        await invoke("export_key", {
            key: key?.id,
            path: destination,
            keyType,
        });
        exportingKey = false;
        revealItemInDir(destination);
    }
</script>

<Dialog.Root
    bind:open={hasKey}
    onOpenChange={(open) => {
        if (!open) {
            key = undefined;
        }
    }}
>
    <form>
        <Dialog.Content class="sm:max-w-106.25">
            <Dialog.Header>
                <Dialog.Description>key details</Dialog.Description>
                <Dialog.Title class="font-bold"
                    >{key?.name}
                    {#if isPrivateKey}<Badge
                            class="bg-blue-500 text-white dark:bg-blue-600 ml-1"
                            ><SquareAsteriskIcon />private</Badge
                        >{/if}</Dialog.Title
                >
            </Dialog.Header>
            <div class="flex flex-col gap-2">
                <Label for="public-key">public key</Label>
                <InputGroup.Root class="mb-4">
                    <InputGroup.Textarea
                        id="public-key"
                        value={key?.contents.public}
                        readonly
                        class="resize-none"
                        wrap="hard"
                    />
                    <InputGroup.Addon align="inline-end" class="h-full">
                        <Button
                            variant="ghost"
                            onclick={() => {
                                navigator.clipboard.writeText(
                                    key?.contents.public || "",
                                );
                                toast.success("copied!");
                            }}
                            class="h-full"><CopyIcon /></Button
                        >
                    </InputGroup.Addon>
                </InputGroup.Root>
                <section id="actions">
                    <Label class="mb-2" for="actions">key actions</Label>
                    <div class="flex flex-row gap-2">
                        <Button class="grow" onclick={encrypt}
                            ><LockIcon /> encrypt</Button
                        >
                        {#if isPrivateKey}<Button class="grow" onclick={decrypt}
                                ><LockOpenIcon />decrypt</Button
                            >{/if}
                    </div>
                </section>
                <section id="manage-key">
                    <Label class="mb-2" for="manage-key">manage key</Label>
                    <div class="flex flex-col gap-2">
                        {#if isPrivateKey}<Button
                                class="grow"
                                variant={"destructive"}
                                onclick={() => (confirming = true)}
                                disabled={exportingKey}
                                >{#if exportingKey}<Spinner /> exporting key...{:else}<FolderUpIcon
                                    /> export private{/if}</Button
                            >{/if}
                        <Button
                            class="grow"
                            onclick={deleteKey}
                            variant={"destructive"}
                            ><TrashIcon /> delete key</Button
                        >
                    </div>
                </section>
            </div>
        </Dialog.Content>
    </form>
</Dialog.Root>
<Confirm
    bind:open={confirming}
    title={"are you sure?"}
    description={"your private key can be used to decrypt any file encrypted to you. if you are looking to receive files, you likely want to send your public key."}
    onaccept={async () => await exportKey("private")}
/>

<style>
    section {
        margin-bottom: 1rem;
    }
</style>
