<script>
    import "../app.css";
    import "./layout.css"
    import { Toaster } from "$lib/components/ui/sonner";
    let { children } = $props();
</script>

<Toaster />
<div id="main-container" class="dark">
    {@render children?.()}
</div>

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
