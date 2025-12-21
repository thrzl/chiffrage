<script lang="ts">
    import * as InputGroup from "$lib/components/ui/input-group/index";
    import { zxcvbn, zxcvbnOptions } from "@zxcvbn-ts/core";
    import type { ZxcvbnResult } from "@zxcvbn-ts/core";
    let {
        password = $bindable(""),
        showMeter = true,
        placeholder = "enter passphrase...",
        strength = $bindable<ZxcvbnResult | null>(),
        oninput: callback,
    }: {
        password: string;
        showMeter?: boolean;
        placeholder?: string;
        strength?: ZxcvbnResult | null;
        oninput?: undefined | ((e: Event) => Promise<void>);
    } = $props();
    let showPassword = $state(false);
    import { EyeIcon, EyeClosedIcon } from "@lucide/svelte";
    import * as zxcvbnCommonPackage from "@zxcvbn-ts/language-common";
    import * as zxcvbnEnPackage from "@zxcvbn-ts/language-en";
    import Progress from "$lib/components/ui/progress/progress.svelte";
    import { andList } from "human-list";

    const options = {
        translations: zxcvbnEnPackage.translations,
        graphs: zxcvbnCommonPackage.adjacencyGraphs,
        dictionary: {
            ...zxcvbnCommonPackage.dictionary,
            ...zxcvbnEnPackage.dictionary,
        },
    };

    zxcvbnOptions.setOptions(options);

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
    console.log("test")
</script>

<InputGroup.Root class={showMeter ? "rounded-b-none" : ""}>
    <InputGroup.Input
        type={showPassword ? "text" : "password"}
        name="pass"
        class={`${showPassword && password.length > 0 ? "font-mono" : ""}`}
        {placeholder}
        bind:value={password}
        oninput={async (e: Event) => {
          strength = zxcvbn(password);
          console.log("hi");
          console.log(callback);
          if (callback) {
              console.log("oninput running");
              await callback(e)
          }}
        }
    />
    <InputGroup.Addon align="inline-end">
        <InputGroup.Button
            variant="ghost"
            onclick={() => (showPassword = !showPassword)}
        >
            {#if showPassword}
                <EyeIcon />{:else}
                <EyeClosedIcon />{/if}
        </InputGroup.Button>
    </InputGroup.Addon>
</InputGroup.Root>
{#if showMeter}
    <Progress
        min={0}
        max={10}
        value={strength ? strength?.guessesLog10 : 0}
        style={`--primary: ${progressColor}; border-top-left-radius: 0px; border-top-right-radius: 0`}
    />
{/if}
<!-- <p>{strength?.feedback?.warning}</p>
<p>
    {strength && strength.feedback && andList(strength?.feedback?.suggestions)}
</p> -->
