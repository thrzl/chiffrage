<script>
    import Button from "$lib/components/ui/button/button.svelte";
    import { EyeClosedIcon, EyeIcon } from "@lucide/svelte";
    import { toast } from "svelte-sonner";
    import { commands, events } from "$lib/bindings";
    let vaultUnlocked = $state(await commands.vaultUnlocked());
    events.vaultStatusUpdate.listen((e) => {
        vaultUnlocked = e.payload === "unlocked";
    });

    async function toggleVault() {
        if (vaultUnlocked) {
            await commands.lockVault();
            toast.success("vault locked successfully");
        } else {
            let result = await commands.authenticate();
            if (result.status === "ok") {
                toast.success("vault unlocked successfully");
                return;
            }
            toast.success("failed to unlock vault", {
                description: result.error,
            });
        }
    }
</script>

<div
    class="position-absolute top-5 right-10 w-min"
    id="vault-badge"
    style="margin: 0"
>
    <Button
        class="rounded-full py-1 line-height-none h-auto pointer-cursor font-mono text-xs"
        variant={vaultUnlocked ? "default" : "outline"}
        onclick={toggleVault}
    >
        {#if vaultUnlocked}
            <EyeIcon /> vault unlocked
        {:else}
            <EyeClosedIcon /> vault locked
        {/if}
    </Button>
</div>

<style>
    #vault-badge {
        position: fixed;
        top: 5;
        right: 5;
        width: min-content;
    }
</style>
