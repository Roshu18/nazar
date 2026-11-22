<script lang="ts">
  import { SvelteMap } from "svelte/reactivity";

  import { measureSizes } from "../shell/shell-api";
  import { countOf } from "../traces/trace-wording";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import LocationRow, { type LocationEntry } from "./LocationRow.svelte";
  import type { LocationReference, ProgramDetails, ProgramLocation } from "./program-api";
  import ProgramHeader from "./ProgramHeader.svelte";
  import {
    containerOf,
    describeLocation,
    formatSize,
    nameOf,
    roleLabel,
  } from "./program-wording";

  type Activity = "idle" | "confirmingDeletion" | "deleting" | "confirmingUninstall" | "uninstalling";

  interface Props {
    details: ProgramDetails;
    isAwaitingUninstaller: boolean;
    onDelete: (locations: readonly LocationReference[]) => Promise<void>;
    onUninstall: () => Promise<void>;
    onCheckAgain: () => void;
    onFailure: (failure: unknown) => void;
  }

  let { details, isAwaitingUninstaller, onDelete, onUninstall, onCheckAgain, onFailure }: Props =
    $props();
  let activity = $state<Activity>("idle");
  let sizes = $state<ReadonlyMap<string, number> | null>(null);
  const selection = new SvelteMap<string, LocationEntry>();

  const groups = $derived([
    { title: "Files and folders", entries: details.fileLocations.map(entryOf) },
    { title: "Registry", entries: details.registryLocations.map(entryOf) },
  ]);
  const entries = $derived(groups.flatMap((group) => group.entries));
  const totalSize = $derived(sizes === null ? null : sizeOf(entries));
  const selectedEntries = $derived(entries.filter((entry) => selection.has(entry.key)));
  const selectedSize = $derived(sizeOf(selectedEntries));
  const likelyCount = $derived(selectedEntries.filter((entry) => !entry.isCertain).length);
  const isEverythingSelected = $derived(
    entries.length > 0 && selectedEntries.length === entries.length,
  );

  $effect(() => {
    const paths = details.fileLocations.map((location) => location.location.path);
    measureSizes(paths).then((measuredSizes) => {
      sizes = new Map(paths.map((path, position) => [path, measuredSizes[position] ?? 0]));
    }, onFailure);
  });

  function entryOf(location: ProgramLocation): LocationEntry {
    const reference = location.location;
    return {
      key: `${reference.kind}|${reference.path}|${reference.valueName ?? ""}`,
      reference,
      name: nameOf(reference),
      container: containerOf(reference),
      roleLabel: roleLabel(location.role),
      isCertain: location.isCertain,
    };
  }

  function sizeOfEntry(entry: LocationEntry): number | null {
    const isMeasured = entry.reference.kind === "fileSystemEntry" && sizes !== null;
    return isMeasured ? (sizes?.get(entry.reference.path) ?? 0) : null;
  }

  function sizeOf(measuredEntries: readonly LocationEntry[]): number {
    return measuredEntries.reduce((total, entry) => total + (sizeOfEntry(entry) ?? 0), 0);
  }

  function toggle(entry: LocationEntry): void {
    if (!selection.delete(entry.key)) {
      selection.set(entry.key, entry);
    }
  }

  function toggleEverything(): void {
    const shouldSelect = !isEverythingSelected;
    selection.clear();
    for (const entry of shouldSelect ? entries : []) {
      selection.set(entry.key, entry);
    }
  }

  async function finish(work: Promise<void>): Promise<void> {
    try {
      await work;
      selection.clear();
    } finally {
      activity = "idle";
    }
  }
</script>

<div class="program">
  <div class="column">
  <ProgramHeader
    {details}
    totalSizeInBytes={totalSize}
    locationCount={entries.length}
    locationSizes={groups[0]?.entries.map((entry) => sizeOfEntry(entry) ?? 0) ?? []}
    isUninstalling={activity === "uninstalling"}
    {isAwaitingUninstaller}
    onUninstall={() => (activity = "confirmingUninstall")}
    {onCheckAgain}
  />

  <div class="toolbar">
    <label class="select-all">
      <input
        class="checkbox"
        type="checkbox"
        checked={isEverythingSelected}
        disabled={entries.length === 0}
        onchange={toggleEverything}
      />
      Select all
    </label>
    <span class="selection-summary">
      {#if selectedEntries.length > 0}
        {selectedEntries.length} selected{selectedSize > 0 ? ` · ${formatSize(selectedSize)}` : ""}
      {/if}
    </span>
    <button
      class="button"
      class:button-destructive={selectedEntries.length > 0}
      disabled={selectedEntries.length === 0 || activity !== "idle"}
      onclick={() => (activity = "confirmingDeletion")}
    >
      {activity === "deleting" ? "Deleting…" : "Delete selected"}
    </button>
  </div>

  <div class="locations">
    {#each groups as group (group.title)}
      {#if group.entries.length > 0}
        <section class="group">
          <h2 class="group-title">{group.title}</h2>
          <ul class="card">
            {#each group.entries as entry (entry.key)}
              <LocationRow
                {entry}
                sizeInBytes={sizeOfEntry(entry)}
                isSelected={selection.has(entry.key)}
                onToggle={() => toggle(entry)}
              />
            {/each}
          </ul>
        </section>
      {/if}
    {/each}
    <p class="footnote">
      Locations marked Likely are matched by the program's name. Check them before you delete them.
    </p>
  </div>
  </div>
</div>

{#if activity === "confirmingDeletion"}
  <ConfirmDialog
    title="Delete {countOf(selectedEntries.length, 'location', 'locations')}?"
    explanation="Files and folders go to the Recycle Bin, so you can bring them back. Registry entries are saved to a backup file first.{selectedSize >
    0
      ? ` This frees about ${formatSize(selectedSize)}.`
      : ''} Windows will ask for administrator permission."
    caution={likelyCount > 0
      ? `${countOf(likelyCount, "location is", "locations are")} matched by name only. Make sure ${likelyCount === 1 ? "it belongs" : "they belong"} to ${details.overview.name}.`
      : undefined}
    items={selectedEntries.map((entry) => describeLocation(entry.reference))}
    confirmLabel="Delete"
    isDestructive={true}
    onConfirm={() => {
      activity = "deleting";
      void finish(onDelete(selectedEntries.map((entry) => entry.reference)));
    }}
    onCancel={() => (activity = "idle")}
  />
{:else if activity === "confirmingUninstall"}
  <ConfirmDialog
    title="Uninstall {details.overview.name}?"
    explanation="Nazar starts the program's own uninstaller. When it finishes, you see what it left behind and can delete that too."
    confirmLabel="Uninstall"
    isDestructive={true}
    onConfirm={() => {
      activity = "uninstalling";
      void finish(onUninstall());
    }}
    onCancel={() => (activity = "idle")}
  />
{/if}

<style>
  .program {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    animation: arrive var(--duration-normal) var(--ease-enter);
  }

  @keyframes arrive {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }

  .column {
    width: 100%;
    max-width: 920px;
    margin: 0 auto;
    padding: 0 var(--space-8) var(--space-8);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) 0;
    border-top: 1px solid var(--color-divider);
    border-bottom: 1px solid var(--color-divider);
  }

  .select-all {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font: var(--text-callout);
  }

  .selection-summary {
    flex: 1;
    font: var(--text-callout);
    font-variant-numeric: tabular-nums;
    color: var(--color-text-secondary);
  }

  .locations {
    padding-top: var(--space-5);
  }

  .group + .group {
    margin-top: var(--space-6);
  }

  .group-title {
    margin: 0 0 var(--space-2) var(--space-1);
    font: var(--text-callout);
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .card {
    margin: 0;
    padding: 0;
    overflow: hidden;
    border: 1px solid var(--color-stroke);
    border-radius: var(--radius-surface);
    background: var(--color-card);
    list-style: none;
  }

  .footnote {
    margin: var(--space-5) var(--space-1) 0;
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }
</style>
