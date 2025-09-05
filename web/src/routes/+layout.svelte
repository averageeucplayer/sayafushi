<script lang="ts">
  import { settings } from "$lib/stores.svelte";
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import "../app.css";
  import { invoke } from "@tauri-apps/api/core";
  import Loader from "./loader.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { listen } from "@tauri-apps/api/event";

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

  let loading = $state(true);
  let label = $state("meter");
  let loadingText = $state("Loading assets...");

  onMount(async () => {
    const loadResult = await invoke<LoadResult>("load");

    const appWindow = getCurrentWebviewWindow();
    label = appWindow.label;
    const updateStatus = loadResult.updateStatus;

    switch(updateStatus.type) {
      case "checking":
        loadingText = "Checking updates...";
        break;
      case "downloading":
        loadingText = `Downloading newest version ${updateStatus.chunk} / ${updateStatus.length}`;
        break;
      case "latest":
        loadingText = "Using latest version";
        loading = false;
        break;
    }

    await listen<UpdateStatus>("on-update", (event) => {
      let updateStatus = event.payload;

      switch(updateStatus.type) {
        case "checking":
          loadingText = "Checking updates...";
          break;
        case "downloading":
          loadingText = `Downloading newest version ${updateStatus.chunk} / ${updateStatus.length}`;
          break;
        case "latest":
          loadingText = "Using latest version";
          loading = false;
          break;
      }
    })
  });

  let { children }: Props = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<div class="{settings.app.general.accentColor} {label === "main" ? loading ? "bg-black" : "" : "bg-black"} text-sm text-white flex items-center justify-center min-h-screen">
  {#if loading}
     <Loader text={loadingText}/>
  {:else}
    <!-- <Loader text={loadingText}/> -->
    {@render children?.()}
  {/if}
</div>
