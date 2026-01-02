<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index";
    import * as Tabs from "$lib/components/ui/tabs/index.js";
    import { Label } from "$lib/components/ui/label/index";
    import { Input } from "$lib/components/ui/input/index";
    import * as Item from "$lib/components/ui/item/index";
    import { Button, buttonVariants } from "$lib/components/ui/button/index";
    import { emit } from "@tauri-apps/api/event";
    import { toast } from "svelte-sonner";
    import Textarea from "$lib/components/ui/textarea/textarea.svelte";
    import ChooseFileButton from "./ChooseFileButton.svelte";
    import { commands } from "$lib/bindings";
    let name = $state("");
    import { bech32 } from "bech32";
    import SlideAlert from "./SlideAlert.svelte";
    let keyFile: string | null = $state(null);
    let keyContent: string | null = $state(null);
    let currentTab: "file" | "paste" = $state("file");
    let { open = $bindable() } = $props();

    async function import_key() {
        if (currentTab === "file") await import_key_file();
        else if (currentTab === "paste") await import_key_text();
    }
    async function import_key_file() {
        if (!name.replaceAll(" ", "")) return toast.error("no name set");
        if (!keyFile) return toast.error("no file selected");

        let keyFileCheck = await commands.checkKeyfileType(keyFile);
        if (
            keyFileCheck.status === "ok" &&
            keyFileCheck.data &&
            !(await commands.vaultUnlocked())
        ) {
            let authComplete = await commands.authenticate();
            if (
                authComplete.status === "error" ||
                authComplete.data === "authenticationCancel"
            )
                return toast.error("authentication failed");
        }
        let keyImport = await commands.importKey(name.trim(), keyFile);
        if (keyImport.status === "error") {
            toast.error("key import failed", { description: keyImport.error });
            return;
        }
        toast.success("imported key");
        emit("update-keys");
        open = false;
        keyFile = null;
        name = "";
        // keys = await invoke("fetch_keys");
    }

    async function import_key_text() {
        if (!name) return toast.error("no name set");
        if (!keyContent) return toast.error("no key content");
        keyContent = keyContent.trim();
        try {
            bech32.decode(keyContent, 1959);
        } catch {
            return toast.error("invalid key", {
                description: "make sure that you copied the correct thing",
            });
        }
        // let keyRaw = keyContent
        //     .toLowerCase()
        //     .replace(/^age/, "")
        //     .replace(/^AGE-SECRET-KEY-/, "");

        if (
            keyContent.startsWith("AGE-SECRET-KEY") &&
            !(await commands.vaultUnlocked())
        ) {
            let authComplete = await commands.authenticate();
            if (
                authComplete.status === "error" ||
                authComplete.data === "authenticationCancel"
            )
                return toast.error("authentication failed");
        }
        let keyImport = await commands.importKeyText(name.trim(), keyContent);
        if (keyImport.status === "error") {
            toast.error("key import failed", { description: keyImport.error });
            return;
        }
        toast.success("imported key");
        emit("update-keys");
        keys = (await commands.fetchKeys()).map((key) => key.name);
        open = false;
        keyContent = "";
        name = "";
        // keys = await invoke("fetch_keys");
    }

    let keys = (await commands.fetchKeys()).map((key) => key.name);

    let alert = $derived.by(async () => {
        if (keys.includes(name)) {
            return {
                title: "key name already in use",
                description: "a key with this name already exists",
            };
        }
        if (currentTab === "paste") {
            if (keyContent) {
                try {
                    bech32.decode(keyContent.trim(), 1959);
                } catch {
                    return {
                        title: "invalid key",
                        description:
                            "the contents of this key cannot be decoded. double-check your paste",
                    };
                }
            }
        }
        if (currentTab === "file" && keyFile) {
            let validation = await commands.validateKeyFile(keyFile);
            if (validation.status === "error") {
                return {
                    title: "invalid key file",
                    description: validation.error,
                };
            }
        }
    });
    let submissionValid = $derived(
        name.replaceAll(" ", "") !== "" &&
            !(currentTab === "file" && !keyFile) &&
            // @ts-ignore 2367
            !(currentTab === "paste" && !keyContent) &&
            !alert,
    );
</script>

<Dialog.Root bind:open>
    <form>
        <Dialog.Content
            class="sm:max-w-106.25"
            onkeydown={async (event) => {
                if (event.key === "Enter") await import_key();
            }}
        >
            <Dialog.Header>
                <Dialog.Title>import key</Dialog.Title>
            </Dialog.Header>
            <div class="grid gap-4">
                <div class="grid gap-3">
                    <Label for="name-1">name</Label>
                    <Input id="name-1" name="name" required bind:value={name} />
                    <Tabs.Root bind:value={currentTab}>
                        <Tabs.List class="w-full">
                            <Tabs.Trigger value="file">file</Tabs.Trigger>
                            <Tabs.Trigger value="paste">paste</Tabs.Trigger>
                        </Tabs.List>
                        <Tabs.Content value="file">
                            <Item.Root variant="outline" class="border-dashed">
                                <Item.Content>
                                    <Item.Title>key file</Item.Title>
                                    <Item.Description
                                        >click here to import a file.</Item.Description
                                    >
                                </Item.Content>
                                <Item.Actions>
                                    <ChooseFileButton
                                        bind:file={keyFile}
                                        bind:name
                                    />
                                </Item.Actions>
                            </Item.Root>
                        </Tabs.Content>
                        <Tabs.Content value="paste">
                            <div
                                class="border-dashed p-4 border text-sm rounded-sm gap-y-1 flex flex-col"
                            >
                                <p class="text-sm leading-snug font-medium">
                                    key text
                                </p>
                                <p class="text-muted-foreground text-sm">
                                    paste your key content
                                </p>
                                <Textarea
                                    bind:value={keyContent}
                                    class="resize-none"
                                    wrap="hard"
                                />
                            </div>
                        </Tabs.Content>
                    </Tabs.Root>
                </div>
            </div>
            <!-- {#await alert then alert} -->
            <SlideAlert bind:alert />
            <!-- {/await} -->
            <Dialog.Footer>
                <Dialog.Close
                    class={buttonVariants({
                        variant: "outline",
                    })}>cancel</Dialog.Close
                >
                <Button
                    class={buttonVariants({
                        variant: "default",
                    })}
                    onclick={import_key}
                    disabled={!submissionValid}>import key</Button
                >
            </Dialog.Footer>
        </Dialog.Content>
    </form>
</Dialog.Root>
