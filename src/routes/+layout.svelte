<script>
    import "../app.css";
    import "./layout.css"
    import { Toaster } from "$lib/components/ui/sonner";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import AppSidebar from "../components/AppSidebar.svelte";
    import {page} from "$app/state"

    let { children } = $props();
</script>

<Toaster richColors position="bottom-center" />
<Sidebar.Provider style="--sidebar-width: 12rem">
    {#if new URL(page.url).pathname !== "/create-vault"}<AppSidebar/>{/if}
<div id="main-container" class="dark">
    {@render children?.()}
</div>
</Sidebar.Provider>

<style>
    #main-container {
        min-height: 100vh;
        width: 100vw;
        display: flex;
        justify-content: center;
        justify-items: center;
        align-content: center;
    }
</style>
