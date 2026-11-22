<script lang="ts" module>
  export type Section = "installed" | "leftovers" | "tracked";
</script>

<script lang="ts">
  interface SectionChoice {
    readonly section: Section;
    readonly label: string;
  }

  interface Props {
    section: Section;
    onChoose: (section: Section) => void;
  }

  const CHOICES: readonly SectionChoice[] = [
    { section: "installed", label: "Installed" },
    { section: "leftovers", label: "Leftovers" },
    { section: "tracked", label: "Tracked" },
  ];

  let { section, onChoose }: Props = $props();
</script>

<div class="switch" role="tablist" aria-label="Which programs to show">
  {#each CHOICES as choice (choice.section)}
    <button
      class="choice"
      class:is-chosen={choice.section === section}
      role="tab"
      aria-selected={choice.section === section}
      onclick={() => onChoose(choice.section)}
    >
      {choice.label}
    </button>
  {/each}
</div>

<style>
  .switch {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius-control);
    background: var(--color-fill);
  }

  .choice {
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    font: var(--text-callout);
    color: var(--color-text-secondary);
    cursor: default;
    transition:
      background var(--duration-fast) var(--ease-enter),
      color var(--duration-fast) linear;
  }

  .choice:hover {
    color: var(--color-text);
  }

  .choice.is-chosen {
    background: var(--color-card);
    color: var(--color-text);
    font-weight: 600;
    box-shadow: var(--shadow-raised);
  }
</style>
