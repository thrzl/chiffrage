<script lang="ts">
    import { TriangleAlertIcon, type IconProps } from "@lucide/svelte";
    import type { Component } from "svelte";
    import * as Alert from "$lib/components/ui/alert/index";
    import { animate } from "motion/mini";
    import type { HTMLAttributes } from "svelte/elements";

    type AlertMsg = { title: string; description: string } | undefined;
    let {
        alert = $bindable(),
        icon: Icon = TriangleAlertIcon,
        class: className = undefined,
        ...restProps
    }: {
        alert: AlertMsg | Promise<AlertMsg>;
        icon?: Component<IconProps, {}, "">;
    } & HTMLAttributes<HTMLDivElement> = $props();
    let alertElement: HTMLDivElement | undefined = $state();

    // @ts-ignore 2345
    $effect(async () => {
        if (!alertElement) return;
        if (alert instanceof Promise) {
            alert = await alert;
        }

        animate(
            alertElement,
            {
                height: `${alert ? alertElement.scrollHeight : "0"}px`,
            },
            { duration: 0.2, ease: "easeOut" },
        ).then(() => {
            if (!alertElement) return;
            let margin = alert ? "0.5rem" : "0rem";
            alertElement.style.marginBottom = margin;
        });
    });
</script>

<div bind:this={alertElement} class="text-left overflow-hidden mb-0 h-0">
    <Alert.Root class={className}>
        <Icon />
        <Alert.Title>{(alert as AlertMsg)?.title}</Alert.Title>
        <Alert.Description>{(alert as AlertMsg)?.description}</Alert.Description>
    </Alert.Root>
</div>
