<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open as openFile } from "@tauri-apps/plugin-dialog";
    import { getFileName } from "$lib/main";
    import * as Dialog from "$lib/components/ui/dialog/index";
    import { Label } from "$lib/components/ui/label/index";
    import { Input } from "$lib/components/ui/input/index";
    import * as Item from "$lib/components/ui/item/index";
    import { Button, buttonVariants } from "$lib/components/ui/button/index";
    import { emit } from "@tauri-apps/api/event";
    import { toast } from "svelte-sonner";
    let name = $state("");
    let keyFile: string | null = $state(null);
    let { open = $bindable() } = $props();

    async function import_key() {
        if (!name) return toast.error("no name set");
        if (!keyFile) return toast.error("no file selected");

        if (!(await invoke("vault_unlocked"))) {
            let authComplete = await invoke("authenticate");
            if (!authComplete) return toast.error("authentication failed")
        }
        await invoke("import_key", { name: name.trim(), path: keyFile });
        toast.success("imported key");
        emit("update-keys");
        open = false;
        keyFile = null;
        name = "";
        // keys = await invoke("fetch_keys");
    }
</script>

<Dialog.Root bind:open={open} onOpenChange={(open) => { if (!open) {keyFile = null; name = ""}}}>
    <form>
        <Dialog.Content class="sm:max-w-[425px]" onkeydown={async (event) => {if (event.key === "Enter") await import_key()}}>
            <Dialog.Header>
                <Dialog.Title>import key</Dialog.Title>
                <Dialog.Description>
                    import a key from a file
                </Dialog.Description>
            </Dialog.Header>
            <div class="grid gap-4">
                <div class="grid gap-3">
                    <Label for="name-1">name</Label>
                    <Input id="name-1" name="name" required bind:value={name} />
                    <Item.Root variant="outline" class="border-dashed">
                        <Item.Content>
                            <Item.Title>key file</Item.Title>
                            <Item.Description
                                >click here to import a file.</Item.Description
                            >
                        </Item.Content>
                        <Item.Actions>
                            <Button
                                variant="outline"
                                size="sm"
                                onclick={async () => {
                                    keyFile = await openFile({
                                        directory: false,
                                        multiple: false,
                                        filters: [
                                            {
                                                name: "age keyfiles",
                                                extensions: [".age", ".txt"],
                                            },
                                        ],
                                    });
                                    if (keyFile && !name) name = getFileName(keyFile)!.split(".").shift()!;
                                }}
                                >{keyFile
                                    ? getFileName(keyFile)
                                    : "choose file"}</Button
                            >
                        </Item.Actions>
                    </Item.Root>
                </div>
            </div>
            <Dialog.Footer>
                <Dialog.Close
                    class={buttonVariants({
                        variant: "outline",
                    })}
                    >cancel</Dialog.Close
                >
                <Button
                    class={buttonVariants({
                        variant: "default",
                    })}
                    onclick={import_key}
                    disabled={name === "" || !keyFile}>import key</Button
                >
            </Dialog.Footer>
        </Dialog.Content>
    </form>
</Dialog.Root>
