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
        AtomIcon,
        TriangleAlertIcon,
    } from "@lucide/svelte";
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import { toast } from "svelte-sonner";
    import * as InputGroup from "$lib/components/ui/input-group/index";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { save } from "@tauri-apps/plugin-dialog";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import { commands, type KeyMetadata } from "$lib/bindings";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import Label from "$lib/components/ui/label/label.svelte";
    let webviewWindow = getCurrentWebviewWindow();
    let { key = $bindable() }: { key: KeyMetadata | undefined } = $props();
    let hasKey = $derived(key !== undefined);
    let confirmation:
        | {
              title: string;
              description: string;
              onaccept: (...args: any[]) => void | Promise<void>;
          }
        | undefined = $state();
    let confirming = $derived(confirmation !== undefined);
    let exportingKey = $state(false);
    let isPrivateKey = $derived(hasKey && key?.key_type === "Private");
    let isPostQuantum = $derived(
        key && key.contents.public.startsWith("age1pq"),
    );

    async function deleteKey() {
        if (!key) return;
        if ((await commands.authenticate()).status !== "ok") {
            toast.error("authentication failed");
            return;
        }
        await commands.deleteKey(key.id);
        webviewWindow.emit("update-keys");
        toast.success("key deleted successfully");
    }

    async function exportKey(keyType: "public" | "private") {
        if (!key) return;
        if (keyType === "private") {
            let authComplete = await commands.authenticate();
            if (authComplete.status !== "ok") {
                toast.error("authentication failed");
                return;
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
        let res = await commands.exportKey(
            key.id,
            destination,
            key.contents.public.startsWith("age1pq") ? "PostQuantum" : "X25519",
        );
        exportingKey = false;
        if (res.status === "ok") {
            toast.success("key exported successfully");
            revealItemInDir(destination);
        } else {
            toast.error(res.error);
        }
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
                        >{/if}
                    {#if isPostQuantum}<Badge
                            class="bg-green-500 text-white dark:bg-green-600 ml-1"
                            ><AtomIcon />quantum-resistant</Badge
                        >{:else}<Badge
                            class="bg-red-500 text-white dark:bg-red-600 ml-1"
                            ><TriangleAlertIcon />quantum-vulnerable</Badge
                        >{/if}</Dialog.Title
                >
            </Dialog.Header>
            <div class="flex flex-col gap-2">
                <Label for="public-key">public key</Label>
                <InputGroup.Root class="mb-4">
                    <InputGroup.Textarea
                        id="public-key"
                        value={key?.contents.public.slice(0, 65) +
                            ((key?.contents.public.length || 0) > 65
                                ? "..."
                                : "")}
                        readonly
                        class="resize-none font-mono"
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
                        <Button
                            class="grow"
                            onclick={() =>
                                (window.location.href = `/encrypt?keys=${key?.id}`)}
                            ><LockIcon /> encrypt</Button
                        >
                        {#if isPrivateKey}<Button
                                class="grow"
                                onclick={() =>
                                    (window.location.href = `/decrypt?key=${key?.id}`)}
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
                                onclick={() =>
                                    (confirmation = {
                                        title: "are you sure?",
                                        description:
                                            "your private key can be used to decrypt every file that has ever been encrypted to you. if you are looking to receive files, you likely want to send your public key.",
                                        onaccept: async () =>
                                            await exportKey("private"),
                                    })}
                                disabled={exportingKey}
                                >{#if exportingKey}<Spinner /> exporting key...{:else}<FolderUpIcon
                                    /> export private{/if}</Button
                            >{/if}
                        <Button
                            class="grow"
                            onclick={() =>
                                (confirmation = {
                                    title: "are you sure?",
                                    description:
                                        "deleting a key is irreversible. there will be no way to restore it unless you have already exported a backup.",
                                    onaccept: deleteKey,
                                })}
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
    title={confirmation?.title || ""}
    description={confirmation?.description || ""}
    onaccept={confirmation?.onaccept || function () {}}
/>

<style>
    section {
        margin-bottom: 1rem;
    }
</style>
