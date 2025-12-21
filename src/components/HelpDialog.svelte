<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index";
    import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
    let { open = $bindable() } = $props();
</script>

<Dialog.Root bind:open>
    <Dialog.Content>
        <Dialog.Header>
            <Dialog.Title>user manual</Dialog.Title>
            <Dialog.Description
                >a short overview of how this works</Dialog.Description
            >
        </Dialog.Header>
        <ScrollArea class="max-h-[60vh]">
            <section>
                <h1>how does age encryption work?</h1>
                <p>
                    age uses <b>asymmetric encryption</b> to allow encrypting
                    files to others. in essence, if
                    <b class="text-orange">alice</b>
                    was trying to encrypt a file to <b>bob</b>:
                </p>
                <ol class="my-4">
                    <li>
                        <b>bob</b> would send
                        <b>alice</b>
                        his <b>public key</b>
                    </li>
                    <li>
                        <b>alice</b> would encrypt the file
                        <b>to bob's public key</b>
                    </li>
                    <li>
                        <b>bob</b> would decrypt it with
                        <b>his own</b> <b>private key</b>
                    </li>
                </ol>
                <p>
                    you can encrypt a file to a <b>public key</b>, and the
                    encrypted output will only be possible to decrypt using the
                    associated <b>private key</b>. <br />
                </p>
                <p class="font-bold">
                    you should never send anyone your private key.
                </p>
            </section>
        </ScrollArea>
    </Dialog.Content></Dialog.Root
>

<style>
    h1 {
        /*@apply font-bold text-lg;*/
        font-size: 1.25rem;
        line-height: 1.75rem;
        font-weight: 700;
        border-bottom: 2px dotted var(--secondary);
        width: max-content;
        margin: 0.5rem 0;
    }
    p {
        font-size: 1rem;
        margin: 0.25rem 0;
    }
    ol li,
    ol {
        list-style-type: decimal;
        list-style-position: inside;
    }
</style>
