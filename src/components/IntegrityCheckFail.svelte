<script lang="ts">
    import * as AlertDialog from "$lib/components/ui/alert-dialog/index";
    import Button from "$lib/components/ui/button/button.svelte";
    import { commands } from "$lib/bindings";
    import { toast } from "svelte-sonner";
    let {
        open = $bindable(),
    }: {
        open: boolean;
    } = $props();
    let wipedKeys = $state(false);
    let keys = await commands.fetchKeys();
    const publicKeys = keys.filter((key) => key.key_type === "Public");
    async function wipePublicKeys() {
        publicKeys.forEach(async (key) => {
            await commands.deleteKey(key.id);
        });
        toast.success(`deleted ${publicKeys.length} public keys.`);
        wipedKeys = true;
    }
</script>

<AlertDialog.Root bind:open>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>integrity check failed</AlertDialog.Title>
            <AlertDialog.Description>
                we could not verify that your vault has not been tampered with.
                your private keys are safe, and therefore decryption is as well.<br
                /><br />however, it's possible your save public keys have been
                altered, which would mean that any file you encrypt may be to
                someone who is not the intended recipient.<br /><br />
                the vault, in its current state, cannot be trusted. it is recommended
                to clear all public keys that are not your own.<br /><br />
                this message will go away next time you make changes to the vault.
            </AlertDialog.Description>
        </AlertDialog.Header>
        {#if publicKeys.length > 0}
            <div class="flex flex-row my-1 gap-2">
                <Button
                    class="flex-1"
                    variant="destructive"
                    disabled={wipedKeys}
                    onclick={wipePublicKeys}>wipe public keys</Button
                >
            </div>
        {/if}
        <AlertDialog.Footer>
            <AlertDialog.Cancel
                onclick={() => {
                    open = false;
                }}>continue</AlertDialog.Cancel
            >
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
