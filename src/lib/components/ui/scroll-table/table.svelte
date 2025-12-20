<script lang="ts">
	import type { ClassValue, HTMLTableAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		height = "auto",
		children,
		containerClass,
		...restProps
	}: WithElementRef<HTMLTableAttributes> & {height: string, containerClass?: ClassValue} = $props();
</script>

<div data-slot="table-container" class={cn("relative w-full overflow-y-auto", containerClass)} style={`max-height: ${height}`}>
	<table
		bind:this={ref}
		data-slot="table"
		class={cn("w-full caption-bottom text-sm", className)}
		{...restProps}
	>
		{@render children?.()}
	</table>
</div>
