<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";

  import type { ProgramOverview } from "../programs/program-api";
  import { formatSize, installedSizeOf, publisherOf } from "../programs/program-wording";
  import type { TraceOverview } from "../traces/trace-api";
  import { countOf, formatStartTime } from "../traces/trace-wording";
  import NavigationItem from "./NavigationItem.svelte";
  import SectionSwitch, { type Section } from "./SectionSwitch.svelte";
  import SortMenu, { type SortOrder } from "./SortMenu.svelte";

  const ICON_SIZE = 16;
  const ICON_STROKE_WIDTH = 1.75;
  const SEARCH_SHORTCUT_KEY = "f";
  const PLACEHOLDER_ROW_COUNT = 8;

  interface Props {
    section: Section;
    programs: readonly ProgramOverview[];
    leftovers: readonly ProgramOverview[];
    isLoadingPrograms: boolean;
    traces: readonly TraceOverview[];
    openItemId: string | null;
    isTracking: boolean;
    onChooseSection: (section: Section) => void;
    onOpenProgram: (id: string) => void;
    onOpenTrace: (name: string) => void;
    onTrack: () => void;
  }

  let {
    section,
    programs,
    leftovers,
    isLoadingPrograms,
    traces,
    openItemId,
    isTracking,
    onChooseSection,
    onOpenProgram,
    onOpenTrace,
    onTrack,
  }: Props = $props();
  let searchText = $state("");
  let sortOrder = $state<SortOrder>("name");
  let searchField = $state<HTMLInputElement | null>(null);

  const comparisons: Record<SortOrder, (first: ProgramOverview, second: ProgramOverview) => number> =
    {
      name: (first, second) =>
        first.name.localeCompare(second.name, undefined, { sensitivity: "base" }),
      size: (first, second) => (second.sizeInKilobytes ?? -1) - (first.sizeInKilobytes ?? -1),
      newest: (first, second) => (second.installDate ?? "").localeCompare(first.installDate ?? ""),
    };
  const shownPrograms = $derived.by(() => {
    const wantedText = searchText.trim().toLowerCase();
    return programs
      .filter((program) =>
        `${program.name} ${program.publisher ?? ""}`.toLowerCase().includes(wantedText),
      )
      .sort(comparisons[sortOrder]);
  });

  function focusSearchOnShortcut(event: KeyboardEvent): void {
    if (event.ctrlKey && event.key.toLowerCase() === SEARCH_SHORTCUT_KEY) {
      event.preventDefault();
      searchField?.focus();
    }
  }

  function sizeLabelOf(program: ProgramOverview): string {
    const size = installedSizeOf(program);
    return size === null ? "" : formatSize(size);
  }
</script>

<svelte:window onkeydown={focusSearchOnShortcut} />

<nav class="pane" aria-label="Programs">
  <div class="controls">
    <SectionSwitch {section} onChoose={onChooseSection} />
    {#if section === "installed"}
      <label class="search">
        <span class="search-icon"><Search size={ICON_SIZE} strokeWidth={ICON_STROKE_WIDTH} /></span>
        <input
          class="search-field"
          type="search"
          placeholder="Search"
          aria-label="Search programs"
          bind:this={searchField}
          bind:value={searchText}
        />
      </label>
    {/if}
  </div>

  {#if section === "installed"}
    <div class="summary">
      <span>
        {isLoadingPrograms ? "Reading programs…" : countOf(shownPrograms.length, "program", "programs")}
      </span>
      <SortMenu {sortOrder} onChoose={(chosenOrder) => (sortOrder = chosenOrder)} />
    </div>
    <ul class="items">
      {#if isLoadingPrograms}
        {#each { length: PLACEHOLDER_ROW_COUNT }, position (position)}
          <li class="placeholder"></li>
        {/each}
      {/if}
      {#each shownPrograms as program (program.id)}
        <NavigationItem
          title={program.name}
          details={publisherOf(program)}
          trailingDetail={sizeLabelOf(program)}
          iconSource={program.iconSource}
          isOpen={program.id === openItemId}
          onOpen={() => onOpenProgram(program.id)}
        />
      {/each}
    </ul>
  {:else if section === "leftovers"}
    <div class="summary">
      <span>{countOf(leftovers.length, "possible leftover", "possible leftovers")}</span>
    </div>
    <ul class="items">
      {#each leftovers as leftover (leftover.id)}
        <NavigationItem
          title={leftover.name}
          details={publisherOf(leftover)}
          trailingDetail=""
          iconSource={leftover.iconSource}
          isOpen={leftover.id === openItemId}
          onOpen={() => onOpenProgram(leftover.id)}
        />
      {/each}
    </ul>
  {:else}
    <div class="summary">
      <span>{countOf(traces.length, "tracked run", "tracked runs")}</span>
      <button
        class="icon-button"
        aria-label="Track a program"
        title="Track a program"
        disabled={isTracking}
        onclick={onTrack}
      >
        <Plus size={ICON_SIZE} strokeWidth={ICON_STROKE_WIDTH} />
      </button>
    </div>
    <ul class="items">
      {#each traces as trace (trace.name)}
        <NavigationItem
          title={trace.programName}
          details={formatStartTime(trace.startedAtUnixSeconds)}
          trailingDetail={countOf(trace.changeCount, "change", "changes")}
          iconSource={trace.programPath}
          isOpen={trace.name === openItemId}
          onOpen={() => onOpenTrace(trace.name)}
        />
      {/each}
    </ul>
  {/if}
</nav>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: var(--space-3) var(--space-2) 0;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: 0 var(--space-2);
    margin-bottom: var(--space-3);
  }

  .search {
    position: relative;
    display: block;
  }

  .search-icon {
    position: absolute;
    top: var(--space-2);
    left: 10px;
    display: grid;
    color: var(--color-text-secondary);
    pointer-events: none;
  }

  .search-field {
    width: 100%;
    height: var(--control-height);
    padding: 0 var(--space-2) 0 var(--space-8);
    border: 0;
    border-radius: var(--radius-control);
    background: var(--color-fill);
    transition: background var(--duration-faster) linear;
  }

  .search-field:hover {
    background: var(--color-fill-hover);
  }

  .search-field::placeholder {
    color: var(--color-text-secondary);
  }

  .summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-3);
    min-height: 28px;
    margin-bottom: var(--space-1);
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }

  .items {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0 0 var(--space-2);
    overflow-y: auto;
    list-style: none;
  }

  .placeholder {
    height: calc(var(--navigation-item-height) - var(--space-2));
    margin: var(--space-1) var(--space-1) var(--space-2);
    border-radius: var(--radius-control);
    background: var(--color-fill);
    animation: breathe 1.2s ease-in-out infinite alternate;
  }

  @keyframes breathe {
    to {
      opacity: 0.4;
    }
  }
</style>
