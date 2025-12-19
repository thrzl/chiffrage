<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import * as Dialog from "$lib/components/ui/dialog/index";
    import { Label } from "$lib/components/ui/label/index";
    import { Input } from "$lib/components/ui/input/index";
    import { buttonVariants } from "$lib/components/ui/button/index";
    import { emit } from "@tauri-apps/api/event";
    let name = $state("");
    let error = $state("");
    let { open } = $props();

    function getName() {
        if (!name) {
            error = "no name set!";
            return;
        }
        return name;
    }

    async function generate_key() {
        if (!getName()) return;
        error = "";
        // Learn more about Tauri commands at https://tauri.app/d,evelop/calling-rust/
        if (!(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        await invoke("generate_keypair", { name: name.trim() });
        emit("update-keys");
        open = false;
    }
</script>

<Dialog.Root bind:open>
    <form>
        <!-- <Dialog.Trigger
            class={buttonVariants({ variant: "default" })}
            >new keypair</Dialog.Trigger
        > -->
        <Dialog.Content class="sm:max-w-[425px]">
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
                    disabled={name === ""}>generate</Dialog.Close
                >
            </Dialog.Footer>
        </Dialog.Content>
    </form>
</Dialog.Root>
