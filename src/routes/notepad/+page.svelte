<script lang="ts">
    import { toast } from "svelte-sonner";
    import { commands } from "$lib/bindings";
    import * as InputGroup from "$lib/components/ui/input-group";
    import * as Select from "$lib/components/ui/select/index";
    import * as Tabs from "$lib/components/ui/tabs/index";

    import Label from "$lib/components/ui/label/label.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import Textarea from "$lib/components/ui/textarea/textarea.svelte";
    import { CopyIcon } from "@lucide/svelte";
    import PasswordBox from "../../components/PasswordBox.svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import { andList } from "human-list";

    let password = $state("");
    let chosenKeys: string[] = $state([]);
    let cryptoMethod: "Scrypt" | "X25519" = $state("X25519");
    let input: string = $state("");
    let output: string = $state("");
    let decryptPossible: boolean = $derived(
        input.startsWith("-----BEGIN AGE ENCRYPTED FILE-----"),
    );
    let processing = $state(false);

    async function decryptText(event: Event) {
        event.preventDefault();
        if (cryptoMethod === "X25519" && !(await commands.vaultUnlocked())) {
            await commands.authenticate();
        }
        processing = true;
        let decryptRes = await commands.decryptText(
            cryptoMethod === "X25519" ? chosenKeys[0] : password,
            input,
            cryptoMethod,
        );
        if (decryptRes.status === "error") {
            let errorText = decryptRes.error.toLowerCase() + ".";
            let description = undefined;
            if (errorText === "header is invalid.") {
                description = `are you sure this is a valid age-encrypted file?`;
            } else if (errorText === "no matching keys found.") {
                description = "incorrect key";
            } else {
                description = errorText;
                errorText = "decryption error";
            }
            toast.error(errorText, { description });
        } else {
            output = decryptRes.data;
        }
        processing = false;
    }
    async function encryptText(event: Event) {
        event.preventDefault();
        processing = true;
        let encryptRes = await commands.encryptText(
            cryptoMethod === "X25519" ? chosenKeys : password,
            input,
        );
        if (encryptRes.status === "error") {
            let errorText = encryptRes.error.toLowerCase() + ".";
            toast.error("encryption error", { description: errorText });
        } else {
            output = encryptRes.data;
        }
        processing = false;
    }
    let keys = $state(await commands.fetchKeys());
    let privateKeys = $derived(
        keys.filter((key) => key.key_type === "Private"),
    );
    let publicKeys = $derived(keys.filter((key) => key.key_type === "Public"));
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
                        type="multiple"
                        name="target key"
                        bind:value={chosenKeys}
                        onValueChange={(value) => {
                            if (decryptPossible) {
                                chosenKeys = [value.reverse()[0]]; // set chosenkeys to just be this
                            }
                        }}
                    >
                        <Select.Trigger class="w-full mb-2">
                            <p>
                                {#if chosenKeys.length > 0}<span
                                        class="font-bold"
                                        >{andList(
                                            chosenKeys.map(
                                                (key) => keyMap[key].name,
                                            ),
                                        )}</span
                                    >{:else}choose identity...{/if}
                            </p>
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Group>
                                <Select.Label>private keys</Select.Label>
                                {#if privateKeys.length > 0}
                                    {#each privateKeys as key}
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
                            {#if !decryptPossible}
                                <Select.Group>
                                    <Select.Label>public keys</Select.Label>
                                    {#if publicKeys.length > 0}
                                        {#each publicKeys as key}
                                            <Select.Item
                                                value={key.id}
                                                label={key.name}
                                            >
                                                {key.name}
                                            </Select.Item>
                                        {/each}
                                    {:else}
                                        <Select.Item
                                            value={"#"}
                                            label={"no key"}
                                            disabled
                                        >
                                            {"no public keys"}
                                        </Select.Item>
                                    {/if}
                                </Select.Group>
                            {/if}
                        </Select.Content>
                    </Select.Root>
                </Tabs.Content>
                <Tabs.Content value="Scrypt" class="grow w-45">
                    <PasswordBox
                        bind:password
                        showMeter={!decryptPossible}
                        showGenerate={!decryptPossible}
                    />
                </Tabs.Content>
            </div>
        </Tabs.Root>
        <Label for="output" class="mt-2">input</Label>
        <Textarea
            id="input"
            class="resize-none mt-2"
            placeholder="your input..."
            bind:value={input}
            oninput={(event) => {
                if (
                    event.currentTarget.value.startsWith(
                        "-----BEGIN AGE ENCRYPTED FILE-----",
                    )
                ) {
                    let firstPrivateKey = chosenKeys.find(
                        (key) => keyMap[key].key_type === "Private",
                    );
                    chosenKeys = firstPrivateKey ? [firstPrivateKey] : [];
                }
            }}
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
                onclick={decryptPossible ? decryptText : encryptText}
                disabled={processing ||
                    (cryptoMethod === "X25519"
                        ? chosenKeys.length
                        : password.length) === 0 ||
                    input.length === 0}
                class="mt-2 grow"
                >{#if processing}<Spinner />
                    {decryptPossible ? "decrypt" : "encrypt"}ing...{:else}{input
                        ? decryptPossible
                            ? "decrypt"
                            : "encrypt"
                        : "encrypt or decrypt"}{/if}</Button
            >
        </div>
    </form>
</main>
