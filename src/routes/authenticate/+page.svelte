<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";

    async function submitPassword() {
        await invoke("authenticate", {
            password: (document.querySelector("input") as HTMLInputElement)
                .value,
        });
        await emit("auth-complete");
        window.close();
    }
    document.addEventListener("keypress", (event) => {
        if (event.key === "Enter") submitPassword();
    });
</script>

<p>hi</p>
<input
    type="password"
    placeholder="enter your vault password"
    onsubmit={submitPassword}
/>
