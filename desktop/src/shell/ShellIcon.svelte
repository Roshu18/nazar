<script lang="ts">
  import Folder from "@lucide/svelte/icons/folder";
  import Package from "@lucide/svelte/icons/package";

  import { iconUrlFor } from "./shell-api";

  const PLACEHOLDER_SHARE_OF_SIZE = 0.62;
  const PLACEHOLDER_STROKE_WIDTH = 1.5;

  interface Props {
    path: string | null;
    size: number;
    placeholder: "program" | "folder";
  }

  let { path, size, placeholder }: Props = $props();
  let url = $state<string | null>(null);

  const Placeholder = $derived(placeholder === "program" ? Package : Folder);

  $effect(() => {
    url = null;
    if (path === null) {
      return;
    }
    let isStillWanted = true;
    void iconUrlFor(path).then((loadedUrl) => {
      if (isStillWanted) {
        url = loadedUrl;
      }
    });
    return () => {
      isStillWanted = false;
    };
  });
</script>

<span class="icon" style:width="{size}px" style:height="{size}px">
  {#if url === null}
    <Placeholder size={size * PLACEHOLDER_SHARE_OF_SIZE} strokeWidth={PLACEHOLDER_STROKE_WIDTH} />
  {:else}
    <img class="image" src={url} alt="" width={size} height={size} />
  {/if}
</span>

<style>
  .icon {
    display: grid;
    flex: none;
    place-items: center;
    color: var(--color-text-secondary);
  }

  .image {
    display: block;
    animation: appear var(--duration-fast) var(--ease-enter);
  }

  @keyframes appear {
    from {
      opacity: 0;
    }
  }
</style>
