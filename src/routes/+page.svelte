<script lang="ts">
  import { onMount } from "svelte";
  import App from "$lib/App.svelte";

  onMount(async () => {
    try {
    } catch (e) {
      console.error("run_backend_command failed:", e);
    }
  });
</script>

<App />