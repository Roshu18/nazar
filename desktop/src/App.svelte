<script lang="ts">
  import { onMount } from "svelte";

  import EmptySelection from "./EmptySelection.svelte";
  import { describeFailure } from "./failure-message";
  import FailureNotice from "./FailureNotice.svelte";
  import NavigationPane from "./navigation/NavigationPane.svelte";
  import type { Section } from "./navigation/SectionSwitch.svelte";
  import DeletionResult from "./programs/DeletionResult.svelte";
  import {
    deleteLocations,
    listInstalledPrograms,
    listLeftovers,
    readProgram,
    uninstallProgram,
    type DeletionReport,
    type LocationReference,
    type ProgramDetails,
    type ProgramOverview,
  } from "./programs/program-api";
  import ProgramView from "./programs/ProgramView.svelte";
  import {
    chooseProgram,
    listTraces,
    readTrace,
    stopTracking,
    trackProgram,
    type TraceDetails,
    type TraceOverview,
  } from "./traces/trace-api";
  import TraceView from "./traces/TraceView.svelte";
  import TrackingNotice from "./tracking/TrackingNotice.svelte";

  type OpenItem =
    | { readonly kind: "program"; readonly details: ProgramDetails }
    | { readonly kind: "trace"; readonly details: TraceDetails };

  let section = $state<Section>("installed");
  let programs = $state<ProgramOverview[]>([]);
  let leftovers = $state<ProgramOverview[]>([]);
  let isLoadingPrograms = $state(true);
  let traces = $state<TraceOverview[]>([]);
  let openItem = $state<OpenItem | null>(null);
  let awaitedUninstallerId = $state<string | null>(null);
  let trackedProgramPath = $state<string | null>(null);
  let failureMessage = $state<string | null>(null);
  let deletionReport = $state<DeletionReport | null>(null);

  const openItemId = $derived(
    openItem === null
      ? null
      : openItem.kind === "program"
        ? openItem.details.overview.id
        : openItem.details.overview.name,
  );

  onMount(() => {
    void showFailureOf(refreshPrograms());
    void showFailureOf(refreshTraces());
  });

  function showFailure(failure: unknown): void {
    failureMessage = describeFailure(failure);
  }

  async function showFailureOf(work: Promise<void>): Promise<void> {
    try {
      await work;
    } catch (failure) {
      showFailure(failure);
    }
  }

  async function refreshPrograms(): Promise<void> {
    try {
      programs = await listInstalledPrograms();
      leftovers = await listLeftovers();
    } finally {
      isLoadingPrograms = false;
    }
  }

  async function refreshTraces(): Promise<void> {
    const listedTraces = await listTraces();
    traces = [...listedTraces].sort(
      (first, second) => second.startedAtUnixSeconds - first.startedAtUnixSeconds,
    );
  }

  async function openProgram(id: string): Promise<void> {
    awaitedUninstallerId = null;
    openItem = { kind: "program", details: await readProgram(id) };
  }

  async function openTrace(name: string): Promise<void> {
    openItem = { kind: "trace", details: await readTrace(name) };
  }

  async function reopenOrCloseAfterChange(programId: string): Promise<ProgramDetails | null> {
    await refreshPrograms();
    const details = await readProgram(programId).catch(() => null);
    openItem = details === null ? null : { kind: "program", details };
    return details;
  }

  async function deleteFromProgram(
    programId: string,
    locations: readonly LocationReference[],
  ): Promise<void> {
    await showFailureOf(
      (async () => {
        deletionReport = await deleteLocations(locations);
        await reopenOrCloseAfterChange(programId);
      })(),
    );
  }

  async function uninstall(programId: string): Promise<void> {
    await showFailureOf(
      (async () => {
        await uninstallProgram(programId);
        const details = await reopenOrCloseAfterChange(programId);
        awaitedUninstallerId = details?.isInstalled === true ? programId : null;
      })(),
    );
  }

  async function checkUninstallAgain(programId: string): Promise<void> {
    const details = await reopenOrCloseAfterChange(programId);
    awaitedUninstallerId = details?.isInstalled === true ? programId : null;
  }

  async function trackChosenProgram(): Promise<void> {
    const programPath = await chooseProgram();
    if (programPath === null) {
      return;
    }
    trackedProgramPath = programPath;
    try {
      const traceName = await trackProgram(programPath);
      await refreshTraces();
      await openTrace(traceName);
    } finally {
      trackedProgramPath = null;
    }
  }
</script>

<div class="window">
  <NavigationPane
    {section}
    {programs}
    {leftovers}
    {isLoadingPrograms}
    {traces}
    {openItemId}
    isTracking={trackedProgramPath !== null}
    onChooseSection={(chosenSection) => (section = chosenSection)}
    onOpenProgram={(id) => showFailureOf(openProgram(id))}
    onOpenTrace={(name) => showFailureOf(openTrace(name))}
    onTrack={() => showFailureOf(trackChosenProgram())}
  />
  <main class="content">
    {#if failureMessage !== null}
      <FailureNotice message={failureMessage} onDismiss={() => (failureMessage = null)} />
    {/if}
    {#if deletionReport !== null}
      <DeletionResult report={deletionReport} onDismiss={() => (deletionReport = null)} />
    {/if}
    {#if trackedProgramPath !== null}
      <TrackingNotice
        programPath={trackedProgramPath}
        onStop={() => showFailureOf(stopTracking())}
      />
    {:else if openItem?.kind === "program"}
      {@const programId = openItem.details.overview.id}
      {#key programId}
        <ProgramView
          details={openItem.details}
          isAwaitingUninstaller={awaitedUninstallerId === programId}
          onDelete={(locations) => deleteFromProgram(programId, locations)}
          onUninstall={() => uninstall(programId)}
          onCheckAgain={() => showFailureOf(checkUninstallAgain(programId))}
          onFailure={showFailure}
        />
      {/key}
    {:else if openItem?.kind === "trace"}
      {#key openItem.details.overview.name}
        <TraceView details={openItem.details} />
      {/key}
    {:else if section === "installed"}
      <EmptySelection
        title="Choose a program"
        explanation="Select a program to see everything that belongs to it: folders, shortcuts, registry entries and how much space they take. Uninstall it, or delete only what you choose."
      />
    {:else if section === "leftovers"}
      <EmptySelection
        title="Leftovers"
        explanation="Folders and registry entries that look like they belong to programs that are no longer installed. Nazar matches them by name, so check each one before you delete it."
      />
    {:else}
      <EmptySelection
        title={traces.length > 0 ? "Choose a tracked run" : "Track a program"}
        explanation="Run an installer, an executable or a script under Nazar. It records every file and registry entry the program creates or changes, including hidden ones."
        actionLabel="Track a program"
        actionNote="Windows will ask for administrator permission."
        onAction={() => showFailureOf(trackChosenProgram())}
      />
    {/if}
  </main>
</div>

<style>
  .window {
    display: grid;
    grid-template-columns: var(--navigation-width) minmax(0, 1fr);
    height: 100%;
  }

  .content {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-top: 1px solid var(--color-stroke);
    border-left: 1px solid var(--color-stroke);
    border-top-left-radius: var(--radius-surface);
    background: var(--color-content);
  }
</style>
