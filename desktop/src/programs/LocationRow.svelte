<script lang="ts" module>
  import type { LocationReference } from "./program-api";

  export interface LocationEntry {
    readonly key: string;
    readonly reference: LocationReference;
    readonly name: string;
    readonly container: string;
    readonly roleLabel: string;
    readonly isCertain: boolean;
  }
</script>

<script lang="ts">
  import FolderKey from "@lucide/svelte/icons/folder-key";
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import Tag from "@lucide/svelte/icons/tag";

  import ShellIcon from "../shell/ShellIcon.svelte";
  import { showInFileManager } from "../shell/shell-api";
  import { formatSize } from "./program-wording";

  const ICON_SIZE = 28;
  const GLYPH_SIZE = 18;
  const GLYPH_STROKE_WIDTH = 1.5;
  const REVEAL_GLYPH_SIZE = 16;

  interface Props {
    entry: LocationEntry;
    sizeInBytes: number | null;
    isSelected: boolean;
    onToggle: () => void;
  }

  let { entry, sizeInBytes, isSelected, onToggle }: Props = $props();

  const isFile = $derived(entry.reference.kind === "fileSystemEntry");
  const RegistryGlyph = $derived(entry.reference.kind === "registryValue" ? Tag : FolderKey);
</script>

<li class="row" class:is-selected={isSelected}>
  <label class="entry">
    <input class="checkbox" type="checkbox" checked={isSelected} onchange={onToggle} />
    {#if isFile}
      <ShellIcon path={entry.reference.path} size={ICON_SIZE} placeholder="folder" />
    {:else}
      <span class="glyph">
        <RegistryGlyph size={GLYPH_SIZE} strokeWidth={GLYPH_STROKE_WIDTH} />
      </span>
    {/if}
    <span class="texts">
      <span class="name" title={entry.name}>{entry.name}</span>
      <span class="container" title={entry.container}>{entry.container}</span>
    </span>
    <span class="role">
      {#if !entry.isCertain}<span class="likely">Likely</span>{/if}
      {entry.roleLabel}
    </span>
    <span class="size">{sizeInBytes === null ? "" : formatSize(sizeInBytes)}</span>
  </label>
  {#if isFile}
    <button
      class="icon-button reveal"
      aria-label="Show {entry.name} in File Explorer"
      title="Show in File Explorer"
      onclick={() => void showInFileManager(entry.reference.path)}
    >
      <FolderSearch size={REVEAL_GLYPH_SIZE} strokeWidth={GLYPH_STROKE_WIDTH} />
    </button>
  {:else}
    <span class="reveal"></span>
  {/if}
</li>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: var(--location-row-height);
    padding: 0 var(--space-3) 0 var(--space-4);
    transition: background var(--duration-faster) linear;
  }

  .row:not(:first-child) {
    box-shadow: inset 0 1px 0 var(--color-divider);
  }

  .row:hover {
    background: var(--color-hover);
  }

  .row.is-selected {
    background: var(--color-selected);
  }

  .entry {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
    min-height: var(--location-row-height);
  }

  .glyph {
    display: grid;
    flex: none;
    place-items: center;
    width: 28px;
    height: 28px;
    color: var(--color-text-secondary);
  }

  .texts {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font: var(--text-body);
    font-weight: 500;
  }

  .container {
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }

  .name,
  .container {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .role {
    flex: none;
    font: var(--text-caption);
    color: var(--color-text-secondary);
    text-align: right;
  }

  .likely {
    margin-right: var(--space-1);
    padding: 2px var(--space-2);
    border-radius: var(--radius-pill);
    background: var(--color-fill);
    color: var(--color-changed);
  }

  .size {
    flex: none;
    width: 72px;
    font: var(--text-callout);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .reveal {
    flex: none;
    width: 28px;
    opacity: 0;
    transition: opacity var(--duration-faster) linear;
  }

  .row:hover .reveal,
  .reveal:focus-visible {
    opacity: 1;
  }
</style>
