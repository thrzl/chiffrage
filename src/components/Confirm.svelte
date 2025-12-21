<script lang="ts">
    import * as AlertDialog from "$lib/components/ui/alert-dialog/index";
    let {
        open = $bindable(false),
        title,
        description,
        onaccept,
    }: {
        open: boolean;
        title: string;
        description: string;
        onaccept: () => any | Promise<any>;
    } = $props();
</script>

<AlertDialog.Root bind:open>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>{title}</AlertDialog.Title>
            <AlertDialog.Description>
                {description}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel onclick={() => (open = false)}
                >cancel</AlertDialog.Cancel
            >
            <AlertDialog.Action
                onclick={() => {
                    open = false;
                    if (onaccept) onaccept();
                }}>continue</AlertDialog.Action
            >
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
