<script lang="ts">
    import "../app.css";
    import "./layout.css"
    import { Toaster } from "$lib/components/ui/sonner";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import AppSidebar from "../components/AppSidebar.svelte";
    import {page} from "$app/state"
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { onMount } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";

    let { children } = $props();
    onMount(async () => {
      let window = getCurrentWebviewWindow()
      await window.show()
    })

    type AgeFileType = "key" | "encryptedFile"
    // @ts-ignore 2345
    listen("file-open", async (path: string) => {
      let fileType: AgeFileType = await invoke("get_file_type", {path});
      switch (fileType) {
        case "key":
          window.location.href = `/?file=${path}`

        case "encryptedFile":
          window.location.href = `/?file=${path}`
      }
    })


</script>

<Toaster richColors />
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
