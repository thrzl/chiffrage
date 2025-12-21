<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import * as Dialog from "$lib/components/ui/dialog/index";
    import { Label } from "$lib/components/ui/label/index";
    import { Input } from "$lib/components/ui/input/index";
    import { buttonVariants } from "$lib/components/ui/button/index";
    import { emit } from "@tauri-apps/api/event";
    import { toast } from "svelte-sonner";
    import type { Key } from "$lib/main";
    import SlideAlert from "./SlideAlert.svelte";
    let name = $state("");
    let { open = $bindable() } = $props();

    async function generate_key() {
        if (!name.replaceAll(" ", "")) return toast.error("no name set");
        if (!(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        await invoke("generate_keypair", { name: name.trim() });
        emit("update-keys");
        open = false;
        toast.success("key generated successfully");
    }

    let keys = ((await invoke("fetch_keys")) as Key[]).map((key) => key.name);
    let alert = $derived.by(() => {
        if (keys.includes(name)) {
            return {
                title: "key name already in use",
                description: "a key with this name already exists",
            };
        }
    });
    let submissionValid = $derived(name.replaceAll(" ", "") !== "" && !alert);
</script>

<Dialog.Root
    bind:open
    onOpenChange={(open) => {
        if (!open) {
            name = "";
        }
    }}
>
    <form>
        <Dialog.Content
            class="sm:max-w-[425px]"
            onkeydown={async (event) => {
                if (event.key === "Enter") await generate_key();
            }}
        >
            <Dialog.Header>
                <Dialog.Title>generate new keypair</Dialog.Title>
                <Dialog.Description>
                    this will generate a public and private key. this action
                    requires authentication in order to encrypt your private
                    key.
                </Dialog.Description>
            </Dialog.Header>
            <div class="grid gap-4">
                <div class="grid gap-3">
                    <Label for="name-1">name</Label>
                    <Input id="name-1" name="name" required bind:value={name} />
                </div>
            </div>
            <SlideAlert bind:alert />
            <Dialog.Footer>
                <Dialog.Close
                    class={buttonVariants({
                        variant: "outline",
                    })}
                    onclick={() => (open = false)}>cancel</Dialog.Close
                >
                <Dialog.Close
                    class={buttonVariants({
                        variant: "default",
                    })}
                    onclick={generate_key}
                    disabled={!submissionValid}>generate</Dialog.Close
                >
            </Dialog.Footer>
        </Dialog.Content>
    </form>
</Dialog.Root>
