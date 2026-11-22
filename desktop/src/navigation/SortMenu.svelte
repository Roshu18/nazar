<script lang="ts" module>
  export type SortOrder = "name" | "size" | "newest";
</script>

<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";

  const GLYPH_SIZE = 14;
  const GLYPH_STROKE_WIDTH = 2;
  const CLOSE_KEY = "Escape";

  interface Choice {
    readonly order: SortOrder;
    readonly label: string;
  }

  interface Props {
    sortOrder: SortOrder;
    onChoose: (sortOrder: SortOrder) => void;
  }

  const CHOICES: readonly Choice[] = [
    { order: "name", label: "Name" },
    { order: "size", label: "Size" },
    { order: "newest", label: "Newest" },
  ];

  let { sortOrder, onChoose }: Props = $props();
  let isOpen = $state(false);
  let container = $state<HTMLDivElement | null>(null);

  const chosenLabel = $derived(
    CHOICES.find((choice) => choice.order === sortOrder)?.label ?? "",
  );

  function choose(chosenOrder: SortOrder): void {
    isOpen = false;
    onChoose(chosenOrder);
  }

  function closeWhenClickedOutside(event: MouseEvent): void {
    const isClickInside = event.target instanceof Node && container?.contains(event.target);
    if (isClickInside !== true) {
      isOpen = false;
    }
  }

  function closeOnEscape(event: KeyboardEvent): void {
    if (event.key === CLOSE_KEY) {
      isOpen = false;
    }
  }
</script>

<svelte:window onclick={closeWhenClickedOutside} onkeydown={closeOnEscape} />

<div class="sort" bind:this={container}>
  <button
    class="trigger"
    aria-haspopup="menu"
    aria-expanded={isOpen}
    aria-label="Sort programs by {chosenLabel}"
    onclick={() => (isOpen = !isOpen)}
  >
    {chosenLabel}
    <ChevronDown size={GLYPH_SIZE} strokeWidth={GLYPH_STROKE_WIDTH} />
  </button>
  {#if isOpen}
    <div class="menu" role="menu">
      {#each CHOICES as choice (choice.order)}
        <button
          class="choice"
          role="menuitemradio"
          aria-checked={choice.order === sortOrder}
          onclick={() => choose(choice.order)}
        >
          <span class="check">
            {#if choice.order === sortOrder}
              <Check size={GLYPH_SIZE} strokeWidth={GLYPH_STROKE_WIDTH} />
            {/if}
          </span>
          {choice.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sort {
    position: relative;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    height: 24px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    font: var(--text-caption);
    color: var(--color-text-secondary);
    cursor: default;
    transition: background var(--duration-faster) linear;
  }

  .trigger:hover,
  .trigger[aria-expanded="true"] {
    background: var(--color-fill);
    color: var(--color-text);
  }

  .menu {
    position: absolute;
    top: calc(100% + var(--space-1));
    right: 0;
    z-index: 10;
    min-width: 136px;
    padding: var(--space-1);
    border: 1px solid var(--color-stroke);
    border-radius: var(--radius-control);
    background: var(--color-menu);
    box-shadow: var(--shadow-menu);
    animation: open var(--duration-fast) var(--ease-enter);
  }

  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }

  .choice {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    height: 28px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    font: var(--text-callout);
    color: var(--color-text);
    text-align: left;
    cursor: default;
  }

  .choice:hover {
    background: var(--color-accent);
    color: var(--color-on-accent);
  }

  .check {
    display: grid;
    flex: none;
    width: 14px;
  }
</style>
