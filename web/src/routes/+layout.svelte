<script lang="ts">
  import { settings } from "$lib/stores.svelte";
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import "../app.css";
  import { invoke } from "@tauri-apps/api/core";
  import Loader from "./loader.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { listen } from "@tauri-apps/api/event";
  import { updateState, setUpdateStatus } from "$lib/update";

  interface Props {
    children?: Snippet;
  }

  type UpdateStatus = 
    { type: "checking" } |
    { type: "latest" } |
    { type: "finished" } |
    { type: "downloading", chunk: number, length: number | null }

  interface LoadResult {
    updateStatus: UpdateStatus;
  }

  let label = $state("meter");

  onMount(async () => {
    const loadResult = await invoke<LoadResult>("load");

    const appWindow = getCurrentWebviewWindow();
    label = appWindow.label;
    const updateStatus = loadResult.updateStatus;
    setUpdateStatus(updateStatus);

    await listen<UpdateStatus>("on-update", (event) => {
      let updateStatus = event.payload;
      setUpdateStatus(updateStatus);
    })
  });

  let { children }: Props = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<div class="{settings.app.general.accentColor} {label === "main" ? $updateState.loading ? "bg-black" : "" : "bg-black"} text-sm text-white flex items-center justify-center min-h-screen">
  {#if $updateState.loading}
     <Loader text={$updateState.text}/>
  {:else}
    {@render children?.()}
  {/if}
</div>
