<script lang="ts">
    import * as InputGroup from "$lib/components/ui/input-group/index";
    import { zxcvbn, zxcvbnOptions } from "@zxcvbn-ts/core";
    import type { ZxcvbnResult } from "@zxcvbn-ts/core";
    import { invoke } from "@tauri-apps/api/core";
    let {
        password = $bindable(""),
        showMeter = true,
        showGenerate = true,
        showCrackTime = true,
        placeholder = "enter passphrase...",
        strength = $bindable<ZxcvbnResult | null>(),
        oninput: callback,
        textAlign = "center"
    }: {
        password: string;
        showMeter?: boolean;
        showGenerate?: boolean;
        showCrackTime?: boolean;
        placeholder?: string;
        strength?: ZxcvbnResult | null;
        oninput?: undefined | ((e: Event) => void | Promise<void>);
        textAlign?: "left" | "center" | "right"
    } = $props();
    let inputElement = $state<HTMLInputElement | null>(null);
    let inputGroupElement = $state<HTMLElement | null>(null);
    import { EyeIcon, EyeClosedIcon, RefreshCcwDotIcon } from "@lucide/svelte";
    import Progress from "$lib/components/ui/progress/progress.svelte";
    import { onMount } from "svelte";
    let showPassword = $state(false);

    const loadOptions = async () => {
      const zxcvbnCommonPackage = await import(
        '@zxcvbn-ts/language-common'
      )
      const zxcvbnEnPackage = await import(
        '@zxcvbn-ts/language-en'
      )

      return {
        dictionary: {
          ...zxcvbnCommonPackage.dictionary,
          ...zxcvbnEnPackage.dictionary,
        },
        graphs: zxcvbnCommonPackage.adjacencyGraphs,
        translations: zxcvbnEnPackage.translations,
      }
    }

    onMount(async () => {
      if (showMeter) {
        zxcvbnOptions.setOptions(await loadOptions())
      }
    })

    let passwordPercentage = $derived(
        strength ? strength.guessesLog10 / 10 : 0,
    );

    let progressColor = $derived(
        passwordPercentage === 0
            ? "white"
            : passwordPercentage > 0.8
              ? "lightgreen"
              : passwordPercentage > 0.5
                ? "yellow"
                : "red",
    );
    async function generatePassphrase() {
      inputElement!.focus()
      password = await invoke("generate_passphrase");
      strength = zxcvbn(password);
      showPassword = true;
    }
    $effect(() => {if (password.length > 3) {strength = zxcvbn(password)} else strength = null})
</script>

<InputGroup.Root bind:ref={inputGroupElement} class={showMeter ? "rounded-b-none" : ""} onfocusout={() => showPassword = false}>
    <InputGroup.Input
        type={showPassword ? "text" : "password"}
        name="pass"
        class={`${showPassword && password.length > 0 ? "font-mono" : ""}`}
        {placeholder}
        bind:value={password}
        bind:ref={inputElement}
        oninput={callback}
    />
    <InputGroup.Addon align="inline-end">
        <InputGroup.Button
            variant="ghost"
            class="show-password"
            onmousedown={(e) => e.preventDefault()}
            onclick={() => {inputElement!.focus(); showPassword = !showPassword}}
        >
            {#if showPassword}
                <EyeIcon />{:else}
                <EyeClosedIcon />{/if}
        </InputGroup.Button>
        {#if showGenerate}<InputGroup.Button
            variant="ghost"
            onclick={generatePassphrase}
            onmousedown={(e) => e.preventDefault()}
            class="generate-password"
        >
            <RefreshCcwDotIcon />
        </InputGroup.Button>{/if}
    </InputGroup.Addon>
</InputGroup.Root>
{#if showMeter}
    <Progress
        min={0}
        max={15}
        value={strength ? strength?.guessesLog10 : 0}
        style={`--primary: ${progressColor}; border-top-left-radius: 0px; border-top-right-radius: 0`}
    />
    {#if showCrackTime}
        <p class="my-2" style={`text-align: ${textAlign}`}>
            {#if strength}
                will take <span class="font-bold"
                    >{strength.crackTimesDisplay
                        .offlineSlowHashing1e4PerSecond}</span
                > to crack
            {:else}enter a password
            {/if}
        </p>
        {/if}
{/if}
