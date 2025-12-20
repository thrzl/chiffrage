<script lang="ts">
	import type { HTMLTableAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		height = "auto",
		children,
		...restProps
	}: WithElementRef<HTMLTableAttributes> & {height: string} = $props();
</script>

<div data-slot="table-container" class="relative w-full overflow-y-auto" style={`height: ${height}`}>
	<table
		bind:this={ref}
		data-slot="table"
		class={cn("w-full caption-bottom text-sm", className)}
		{...restProps}
	>
		{@render children?.()}
	</table>
</div>
