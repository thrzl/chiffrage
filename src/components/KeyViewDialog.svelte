<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index";
    import Button from "$lib/components/ui/button/button.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { ask, open, save } from "@tauri-apps/plugin-dialog";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import { Channel } from "@tauri-apps/api/core";
    import type { Key } from "$lib/main";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
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
            error = await invoke("authenticate");
            if (error) return alert(error);
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
                <Dialog.Title>view key {key?.name}</Dialog.Title>
                <Dialog.Description>
                    this will generate a public and private key. this action
                    requires authentication in order to encrypt your private
                    key.
                </Dialog.Description>
            </Dialog.Header>
            <p>has private key? {isPrivateKey ? "yes" : "no"}</p>
            <nav>
                <Button onclick={encrypt}>encrypt</Button>
                {#if isPrivateKey}<Button onclick={decrypt}>decrypt</Button
                    >{/if}
                <br />
                <h2>manage key</h2>
                <Button onclick={deleteKey}>delete key</Button>
                <Button onclick={() => exportKey("public")}
                    >export public</Button
                >
                {#if isPrivateKey}<Button onclick={() => exportKey("private")}
                        >export private</Button
                    >{/if}
            </nav>
        </Dialog.Content>
    </form>
</Dialog.Root>
