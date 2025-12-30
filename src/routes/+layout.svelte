<script>
    import "../app.css";
    import "./layout.css"
    import { Toaster } from "$lib/components/ui/sonner";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import AppSidebar from "../components/AppSidebar.svelte";
    import VaultAuthStatus from "../components/VaultAuthStatus.svelte"
    import {page} from "$app/state"
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { onMount } from "svelte";
    import IntegrityCheckFail from "../components/IntegrityCheckFail.svelte";
    import { events } from "$lib/bindings";

    let integrityCheckFailed = $state(false);
    events.vaultStatusUpdate.listen((e) => {
      if (e.payload === "verificationFail") {
        integrityCheckFailed = true
      }
    })

    let { children } = $props();
    onMount(async () => {
      let window = getCurrentWebviewWindow()
      await window.show()
    })
</script>

<Toaster richColors />
{#if page.route.id !== "/create-vault" && page.route.id !== "/unlock"}
<VaultAuthStatus />
{/if}
<Sidebar.Provider style="--sidebar-width: 12rem">
    {#if page.route.id !== "/create-vault"}<AppSidebar/>{/if}
<div id="main-container" class="dark">
    {@render children?.()}
</div>
</Sidebar.Provider>
<IntegrityCheckFail bind:open={integrityCheckFailed}/>

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
