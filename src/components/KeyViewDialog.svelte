<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index";
    import Button from "$lib/components/ui/button/button.svelte";
    import SquareAsterisk from "@lucide/svelte/icons/square-asterisk";
    import FolderUp from "@lucide/svelte/icons/folder-up";
    import Trash from "@lucide/svelte/icons/trash";
    import Lock from "@lucide/svelte/icons/lock";
    import Unlock from "@lucide/svelte/icons/lock-open";
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import { toast } from "svelte-sonner";
    import Textarea from "$lib/components/ui/textarea/textarea.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { ask, open, save } from "@tauri-apps/plugin-dialog";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import { Channel } from "@tauri-apps/api/core";
    import type { Key } from "$lib/main";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import Label from "$lib/components/ui/label/label.svelte";
    let webviewWindow = getCurrentWebviewWindow();
    let name = $state("");
    let error = $state("");
    let { key = $bindable() }: { key: Key | undefined } = $props();
    let hasKey = $derived(key !== undefined);
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
        await invoke("authenticate");
        await invoke("delete_key", { id: key?.id });
        webviewWindow.emit("update-keys");
        await webviewWindow.close();
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
                error = "authentication failed";
                return toast.error("authentication failed");
            }
        }
        const destination = await save({
            filters: [{ name: "age key file", extensions: ["age"] }],
        });
        if (!destination) {
            return;
        }
        await invoke("export_key", {
            key: key?.id,
            path: destination,
            keyType,
        });
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
        <!-- <Dialog.Trigger
            class={buttonVariants({ variant: "default" })}
            >new keypair</Dialog.Trigger
        > -->
        <Dialog.Content class="sm:max-w-[425px]">
            <Dialog.Header>
                <Dialog.Description>key details</Dialog.Description>
                <Dialog.Title class="font-bold"
                    >{key?.name}
                    {#if isPrivateKey}<Badge
                            class="bg-blue-500 text-white dark:bg-blue-600"
                            ><SquareAsterisk />private</Badge
                        >{/if}</Dialog.Title
                >
            </Dialog.Header>
            <div class="flex flex-col gap-2">
                <section>
                    <Label class="mb-2" for="public-key">public key</Label>
                    <Textarea
                        id="public-key"
                        value={key?.contents.public}
                        onclick={() => {
                            navigator.clipboard.writeText(
                                key?.contents.public || "",
                            );
                            toast.success("copied!");
                        }}
                        readonly
                        class="resize-none"
                    />
                </section>
                <section id="actions">
                    <Label class="mb-2" for="actions">key actions</Label>
                    <div class="flex flex-row gap-2">
                        <Button class="grow" onclick={encrypt}
                            ><Lock /> encrypt</Button
                        >
                        {#if isPrivateKey}<Button class="grow" onclick={decrypt}
                                ><Unlock />decrypt</Button
                            >{/if}
                    </div>
                </section>
                <section id="manage-key">
                    <Label class="mb-2" for="manage-key">manage key</Label>
                    <div class="flex flex-col gap-2">
                        {#if isPrivateKey}<Button
                                class="grow"
                                variant={"destructive"}
                                onclick={() => exportKey("private")}
                                ><FolderUp /> export private</Button
                            >{/if}
                        <Button
                            class="grow"
                            onclick={deleteKey}
                            variant={"destructive"}><Trash /> delete key</Button
                        >
                    </div>
                </section>
            </div>
        </Dialog.Content>
    </form>
</Dialog.Root>

<style>
    section {
        margin-bottom: 1rem;
    }
</style>
