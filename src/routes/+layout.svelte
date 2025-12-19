<script>
    import "../app.css";
    import "./layout.css"
    import { Toaster } from "$lib/components/ui/sonner";
      import * as Sidebar from "$lib/components/ui/sidebar/index.js";
      import AppSidebar from "../components/AppSidebar.svelte";

    let { children } = $props();
</script>

<Toaster />
<Sidebar.Provider style="--sidebar-width: 12rem">
    <AppSidebar/>
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
