<script lang="ts">
  import EntityHeader from "../layout/EntityHeader.svelte";
  import FactStrip, { type Fact } from "../layout/FactStrip.svelte";
  import { countOf } from "../traces/trace-wording";
  import type { ProgramDetails } from "./program-api";
  import { formatInstallDate, formatSize, publisherOf } from "./program-wording";

  const SHARE_OPACITY_STEP = 0.28;
  const FAINTEST_SHARE_OPACITY = 0.2;

  interface Props {
    details: ProgramDetails;
    totalSizeInBytes: number | null;
    locationCount: number;
    locationSizes: readonly number[];
    isUninstalling: boolean;
    isAwaitingUninstaller: boolean;
    onUninstall: () => void;
    onCheckAgain: () => void;
  }

  let {
    details,
    totalSizeInBytes,
    locationCount,
    locationSizes,
    isUninstalling,
    isAwaitingUninstaller,
    onUninstall,
    onCheckAgain,
  }: Props = $props();

  const program = $derived(details.overview);
  const facts = $derived.by(() => {
    const size = totalSizeInBytes === null ? "Measuring…" : formatSize(totalSizeInBytes);
    const installDate = formatInstallDate(program.installDate);
    const possibleFacts: readonly (Fact | null)[] = [
      { label: "On disk", value: size },
      program.version === null ? null : { label: "Version", value: program.version },
      installDate === null ? null : { label: "Installed", value: installDate },
      { label: "Locations", value: String(locationCount) },
    ];
    return possibleFacts.filter((fact) => fact !== null);
  });
  const status = $derived.by(() => {
    if (isUninstalling) {
      return "Finish the steps in the uninstaller's window.";
    }
    if (details.isInstalled) {
      return isAwaitingUninstaller
        ? "The uninstaller closed, but the program is still installed. If its window is still open, finish it first."
        : "";
    }
    return locationCount === 0
      ? "Not installed. Nothing remains."
      : `Not installed. ${countOf(locationCount, "location remains", "locations remain")}.`;
  });
  const shares = $derived(
    locationSizes.filter((size) => size > 0).sort((first, second) => second - first),
  );

  function opacityOf(position: number): number {
    return Math.max(FAINTEST_SHARE_OPACITY, 1 - position * SHARE_OPACITY_STEP);
  }
</script>

<header class="header">
  <EntityHeader iconSource={program.iconSource} title={program.name} subtitle={publisherOf(program)}>
    {#if isAwaitingUninstaller}
      <button class="button" onclick={onCheckAgain}>Check again</button>
    {:else if details.isInstalled}
      <button
        class="button button-primary"
        disabled={!program.hasUninstaller || isUninstalling}
        title={program.hasUninstaller ? "" : "This program does not say how to uninstall it."}
        onclick={onUninstall}
      >
        {isUninstalling ? "Uninstalling…" : "Uninstall"}
      </button>
    {/if}
  </EntityHeader>

  {#if status !== ""}
    <p class="status" class:is-done={!details.isInstalled && locationCount === 0}>{status}</p>
  {/if}

  <FactStrip {facts} />

  {#if shares.length > 1}
    <div class="storage" role="img" aria-label="How the space is divided between the locations">
      {#each shares as share, position (position)}
        <span class="share" style:flex-grow={share} style:opacity={opacityOf(position)}></span>
      {/each}
    </div>
  {/if}
</header>

<style>
  .header {
    padding: var(--space-8) 0 var(--space-5);
  }

  .status {
    margin: var(--space-4) 0 0;
    color: var(--color-text-secondary);
  }

  .status.is-done {
    color: var(--color-created);
  }

  .storage {
    display: flex;
    gap: 2px;
    height: 6px;
    margin-top: var(--space-5);
    overflow: hidden;
    border-radius: var(--radius-pill);
  }

  .share {
    min-width: 6px;
    background: var(--color-accent-text);
  }
</style>
