<script>
    import "../app.css";
    import "./layout.css"
    let { children } = $props();
</script>

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
