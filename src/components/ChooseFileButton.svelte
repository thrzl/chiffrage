<script lang="ts">
    let {
        file = $bindable(),
        name = $bindable(),
        onchoose,
    }: {
        file: string | null;
        name?: string | undefined;
        onchoose?: () => void | Promise<void>;
    } = $props();
    import Button from "$lib/components/ui/button/button.svelte";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { getFileName } from "$lib/main";
    let choosing = $state(false);
    const chooseFile = async () => {
        choosing = true;
        await new Promise((resolve) => setTimeout(resolve, 50)); // for feedback
        file = await open({
            directory: false,
            multiple: false,
            filters: [
                {
                    name: "age keyfiles",
                    extensions: [".age", ".txt"],
                },
            ],
        });
        if (file && !name) name = getFileName(file)!.split(".").shift()!;
        choosing = false;
        if (onchoose) onchoose();
    };
</script>

<Button
    variant="outline"
    size="sm"
    onclick={chooseFile}
    class="truncate w-24"
    disabled={choosing}
    >{#if choosing}<Spinner /> opening{:else}{file
            ? getFileName(file)
            : "choose file"}{/if}</Button
>
