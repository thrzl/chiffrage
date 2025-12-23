<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "svelte-sonner";
    import type { Key } from "$lib/main";
    import * as InputGroup from "$lib/components/ui/input-group";
    import * as Select from "$lib/components/ui/select/index";
    import * as Tabs from "$lib/components/ui/tabs/index";

    import Label from "$lib/components/ui/label/label.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import Textarea from "$lib/components/ui/textarea/textarea.svelte";
    import { CopyIcon } from "@lucide/svelte";
    import PasswordBox from "../../components/PasswordBox.svelte";

    let password = $state("");
    let chosenKey = $state(
        new URLSearchParams(window.location.search).get("key") ?? "",
    );
    let cryptoMethod: "Scrypt" | "X25519" = $state("X25519");
    let input: string = $state("");
    let output: string = $state("");
    let decryptPossible: boolean = $derived(
        await invoke("armor_check_text", {
            text: input,
        }),
    );

    async function decryptText(event: Event) {
        event.preventDefault();
        if (cryptoMethod === "X25519" && !(await invoke("vault_unlocked"))) {
            await invoke("authenticate");
        }
        try {
            output = await invoke("decrypt_text", {
                privateKey: cryptoMethod === "X25519" ? chosenKey : password,
                text: input,
                method: cryptoMethod,
            });
        } catch (e) {
            let errorText = (e as string).toLowerCase() + ".";
            let description = undefined;
            if (errorText === "header is invalid.") {
                description = `are you sure this is a valid age-encrypted file?`;
            } else {
                description = errorText;
                e = "decryption error";
            }
            toast.error(errorText, { description });
        }
    }
    async function encryptText(event: Event) {
        event.preventDefault();
        try {
            output = await invoke("encrypt_text", {
                recipient: cryptoMethod === "X25519" ? chosenKey : password,
                text: input,
            });
            console.log("text encrypted:", output);
        } catch (e) {
            let errorText = (e as string).toLowerCase() + ".";
            toast.error("encryption error", { description: errorText });
        }
    }
    let keyFetch: Key[] = $state(await invoke("fetch_keys"));
    let keys = keyFetch.filter((key) => key.key_type === "Private");
    let keyMap = $derived(Object.fromEntries(keys.map((key) => [key.id, key])));
</script>

<main class="container">
    <h1 class="text-2xl font-bold mb-2">notepad</h1>

    <form class="w-4/5 mx-auto flex flex-col">
        <Tabs.Root bind:value={cryptoMethod}
            ><Tabs.List class="w-full">
                <Tabs.Trigger value="X25519">keys</Tabs.Trigger>
                <Tabs.Trigger value="Scrypt">passphrase</Tabs.Trigger>
            </Tabs.List>
            <div
                class="flex flex-row gap-2 justify-items-center justify-center mx-auto w-full"
            >
                <Tabs.Content value="X25519" class="grow w-45">
                    <Select.Root
                        type="single"
                        name="target key"
                        bind:value={chosenKey}
                    >
                        <Select.Trigger class="w-full mb-2">
                            <p>
                                {#if chosenKey}<span class="font-bold"
                                        >{keyMap[chosenKey].name}</span
                                    >{:else}choose recipient...{/if}
                            </p>
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Group>
                                <Select.Label>private keys</Select.Label>
                                {#if keys.length > 0}
                                    {#each keys as key}
                                        <Select.Item
                                            value={key.id}
                                            label={key.name}
                                        >
                                            {key.name}
                                        </Select.Item>
                                    {/each}
                                {/if}
                                {#if keys.length === 0}
                                    <Select.Item
                                        value={"#"}
                                        label={"no key"}
                                        disabled
                                    >
                                        {"no private keys"}
                                    </Select.Item>
                                {/if}
                            </Select.Group>
                        </Select.Content>
                    </Select.Root>
                </Tabs.Content>
                <Tabs.Content value="Scrypt" class="grow w-45">
                    <PasswordBox bind:password />
                </Tabs.Content>
            </div>
        </Tabs.Root>
        <Label for="output" class="mt-2">input</Label>
        <Textarea
            id="input"
            class="resize-none mt-2"
            placeholder="your input..."
            bind:value={input}
        />
        <Label for="output" class="mt-4">output</Label>
        <InputGroup.Root class="mt-2 bg-card!">
            <InputGroup.Textarea
                id="output"
                bind:value={output}
                placeholder="your output"
                readonly
                class="resize-none"
                wrap="hard"
            />
            <InputGroup.Addon align="inline-end" class="h-full">
                <Button
                    variant="ghost"
                    onclick={() => {
                        navigator.clipboard.writeText(output);
                        toast.success("copied!");
                    }}
                    class="h-full"><CopyIcon /></Button
                >
            </InputGroup.Addon>
        </InputGroup.Root>
        <div class="flex-row flex gap-2">
            <Button
                onclick={encryptText}
                disabled={(cryptoMethod === "X25519"
                    ? chosenKey.length
                    : password.length) === 0 || !input}
                class="mt-2 grow">encrypt</Button
            >
            <Button
                onclick={decryptText}
                disabled={(cryptoMethod === "X25519"
                    ? chosenKey.length
                    : password.length) === 0 ||
                    !input ||
                    !decryptPossible}
                class="mt-2 grow">decrypt</Button
            >
        </div>
    </form>
</main>
