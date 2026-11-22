<script lang="ts">
  import EntityHeader from "../layout/EntityHeader.svelte";
  import FactStrip, { type Fact } from "../layout/FactStrip.svelte";
  import { buildLocationTree, type LocationRow } from "../locations/location-tree";
  import LocationTree from "../locations/LocationTree.svelte";
  import TabStrip, { type Tab } from "../locations/TabStrip.svelte";
  import RemovalPlan from "./RemovalPlan.svelte";
  import type { ChangeRow, TraceDetails } from "./trace-api";
  import { attributionNote, countOf, effectLabel, formatStartTime } from "./trace-wording";

  interface Props {
    details: TraceDetails;
  }

  let { details }: Props = $props();
  let openTabName = $state("files");

  const fileTree = $derived(buildLocationTree(details.fileChanges.map(locationRowOf)));
  const registryTree = $derived(buildLocationTree(details.registryChanges.map(locationRowOf)));
  const tabs = $derived<readonly Tab[]>([
    { name: "files", label: "Files", count: details.fileChanges.length },
    { name: "registry", label: "Registry", count: details.registryChanges.length },
    { name: "removal", label: "Removal plan", count: details.removalSteps.length },
  ]);
  const facts = $derived<readonly Fact[]>([
    { label: "Changes", value: String(details.overview.changeCount) },
    { label: "Files and folders", value: String(details.fileChanges.length) },
    { label: "Registry entries", value: String(details.registryChanges.length) },
    { label: "Tracked", value: formatStartTime(details.overview.startedAtUnixSeconds) },
    { label: "Duration", value: countOf(details.durationInSeconds, "second", "seconds") },
  ]);

  function locationRowOf(change: ChangeRow): LocationRow {
    return {
      path: change.path,
      valueName: change.valueName,
      label: effectLabel(change.effect),
      tone: change.effect,
      note: attributionNote(change.attribution),
    };
  }
</script>

<div class="trace">
  <div class="column">
    <header class="header">
      <EntityHeader
        iconSource={details.overview.programPath}
        title={details.overview.programName}
        subtitle={details.overview.programPath}
      />
      <FactStrip {facts} />
    </header>

    <TabStrip {tabs} {openTabName} onOpen={(tabName) => (openTabName = tabName)} />

    <div class="card" role="tabpanel">
      {#if openTabName === "files"}
        <LocationTree
          nodes={fileTree}
          namespace="files"
          emptyMessage="This program did not create or change any files."
        />
      {:else if openTabName === "registry"}
        <LocationTree
          nodes={registryTree}
          namespace="registry"
          emptyMessage="This program did not create or change any registry entries."
        />
      {:else}
        <RemovalPlan steps={details.removalSteps} />
      {/if}
    </div>

    {#if details.unresolvedEventCount > 0}
      <p class="footnote">
        {countOf(details.unresolvedEventCount, "registry write", "registry writes")} could not be matched
        to a key and may be missing from this list.
      </p>
    {/if}
  </div>
</div>

<style>
  .trace {
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

  .header {
    padding: var(--space-8) 0 var(--space-6);
  }

  .card {
    margin-top: var(--space-4);
    padding: var(--space-2);
    border: 1px solid var(--color-stroke);
    border-radius: var(--radius-surface);
    background: var(--color-card);
  }

  .footnote {
    margin: var(--space-4) var(--space-1) 0;
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }
</style>
