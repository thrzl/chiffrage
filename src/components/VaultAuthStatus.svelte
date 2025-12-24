<script>
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import { EyeClosedIcon, EyeIcon } from "@lucide/svelte";
    import { toast } from "svelte-sonner";
    let { vaultUnlocked = $bindable(false) } = $props();
</script>

<div
    class="position-absolute top-5 right-10 w-min"
    id="vault-badge"
    style="margin: 0"
>
    <Button
        class="rounded-full py-1 line-height-none h-auto pointer-cursor"
        variant={vaultUnlocked ? "default" : "outline"}
        onclick={() => (vaultUnlocked = !vaultUnlocked)}
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
        right: 5
        width: min-content
    }
</style>
