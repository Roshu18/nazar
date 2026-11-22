<script lang="ts">
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import File from "@lucide/svelte/icons/file";
  import Folder from "@lucide/svelte/icons/folder";
  import Tag from "@lucide/svelte/icons/tag";
  import { untrack } from "svelte";

  import type { LocationNamespace, LocationTreeNode } from "./location-tree";
  import LocationTreeBranch from "./LocationTreeBranch.svelte";

  const INITIALLY_EXPANDED_DEPTH = 3;
  const ICON_SIZE = 16;
  const ICON_STROKE_WIDTH = 1.5;
  const FILE_EXTENSION_PATTERN = /\.[a-z0-9]{1,5}$/i;

  interface Props {
    node: LocationTreeNode;
    namespace: LocationNamespace;
    depth: number;
    isSelected?: ((identity: string) => boolean) | undefined;
    onToggle?: ((node: LocationTreeNode) => void) | undefined;
  }

  let { node, namespace, depth, isSelected, onToggle }: Props = $props();
  let isExpanded = $state(untrack(() => depth) < INITIALLY_EXPANDED_DEPTH);

  const hasChildren = $derived(node.children.length > 0);
  const looksLikeFile = $derived(
    namespace === "files" && !hasChildren && FILE_EXTENSION_PATTERN.test(node.label),
  );
  const Icon = $derived(node.kind === "value" ? Tag : looksLikeFile ? File : Folder);
  const isSelectable = $derived(isSelected !== undefined && onToggle !== undefined);
  const canBeSelected = $derived(isSelectable && node.row !== null);
  const isRowSelected = $derived(canBeSelected && (isSelected?.(node.identity) ?? false));
</script>

<li role="treeitem" aria-selected={isRowSelected} aria-expanded={hasChildren ? isExpanded : undefined}>
  <svelte:element this={canBeSelected ? "label" : "div"} class="row" class:is-selected={isRowSelected}>
    {#if isSelectable}
      <span class="selection">
        {#if node.row !== null}
          <input
            class="checkbox"
            type="checkbox"
            checked={isRowSelected}
            onchange={() => onToggle?.(node)}
          />
        {/if}
      </span>
    {/if}
    <span class="indentation" style:width="calc({depth} * var(--tree-indentation))"></span>
    {#if hasChildren}
      <button
        class="toggle"
        class:is-expanded={isExpanded}
        aria-label={isExpanded ? "Collapse" : "Expand"}
        onclick={() => (isExpanded = !isExpanded)}
      >
        <ChevronRight size={ICON_SIZE} strokeWidth={ICON_STROKE_WIDTH} />
      </button>
    {:else}
      <span class="toggle"></span>
    {/if}
    <span class="icon"><Icon size={ICON_SIZE} strokeWidth={ICON_STROKE_WIDTH} /></span>
    <span class="label" title={node.label}>{node.label}</span>
    {#if node.row !== null}
      <span class="note">{node.row.note}</span>
      <span class="tag tone-{node.row.tone}">{node.row.label}</span>
    {/if}
  </svelte:element>

  {#if hasChildren && isExpanded}
    <ul class="children" role="group">
      {#each node.children as child (child.identity)}
        <LocationTreeBranch node={child} {namespace} {isSelected} {onToggle} depth={depth + 1} />
      {/each}
    </ul>
  {/if}
</li>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: var(--row-height);
    padding: 0 var(--space-3) 0 var(--space-2);
    border-radius: var(--radius-control);
  }

  .row:hover {
    background: var(--color-hover);
  }

  .row.is-selected {
    background: var(--color-selected);
  }

  .selection {
    display: grid;
    flex: none;
    width: var(--icon-size);
    margin-right: var(--space-1);
  }

  .indentation {
    flex: none;
    margin-right: calc(var(--space-2) * -1);
  }

  .toggle {
    display: grid;
    place-items: center;
    flex: none;
    width: var(--icon-size);
    height: var(--icon-size);
    padding: 0;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text-secondary);
    transition: transform 120ms ease-out;
  }

  .toggle.is-expanded {
    transform: rotate(90deg);
  }

  .checkbox {
    flex: none;
    width: var(--icon-size);
    height: var(--icon-size);
    margin: 0;
    accent-color: var(--color-accent);
  }

  .icon {
    display: grid;
    flex: none;
    color: var(--color-text-secondary);
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note {
    flex: none;
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }

  .tag {
    flex: none;
    min-width: 64px;
    font: var(--text-caption);
    text-align: right;
  }

  .tone-neutral {
    color: var(--color-text-secondary);
  }

  .tone-created {
    color: var(--color-created);
  }

  .tone-modified {
    color: var(--color-changed);
  }

  .tone-deleted {
    color: var(--color-deleted);
  }

  .children {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .toggle {
      transition: none;
    }
  }
</style>
