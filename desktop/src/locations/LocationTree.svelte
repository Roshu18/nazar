<script lang="ts">
  import type { LocationNamespace, LocationTreeNode } from "./location-tree";
  import LocationTreeBranch from "./LocationTreeBranch.svelte";

  const TOP_LEVEL_DEPTH = 0;

  interface Props {
    nodes: readonly LocationTreeNode[];
    namespace: LocationNamespace;
    emptyMessage: string;
    isSelected?: ((identity: string) => boolean) | undefined;
    onToggle?: ((node: LocationTreeNode) => void) | undefined;
  }

  let { nodes, namespace, emptyMessage, isSelected, onToggle }: Props = $props();
</script>

{#if nodes.length === 0}
  <p class="empty">{emptyMessage}</p>
{:else}
  <ul class="tree" role="tree">
    {#each nodes as node (node.identity)}
      <LocationTreeBranch {node} {namespace} {isSelected} {onToggle} depth={TOP_LEVEL_DEPTH} />
    {/each}
  </ul>
{/if}

<style>
  .tree {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .empty {
    margin: var(--space-4) var(--space-2);
    color: var(--color-text-secondary);
  }
</style>
