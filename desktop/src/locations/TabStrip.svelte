<script lang="ts" module>
  export interface Tab {
    readonly name: string;
    readonly label: string;
    readonly count: number;
  }
</script>

<script lang="ts">
  interface Props {
    tabs: readonly Tab[];
    openTabName: string;
    onOpen: (tabName: string) => void;
  }

  let { tabs, openTabName, onOpen }: Props = $props();
</script>

<div class="tabs" role="tablist">
  {#each tabs as tab (tab.name)}
    <button
      class="tab"
      class:is-open={tab.name === openTabName}
      role="tab"
      aria-selected={tab.name === openTabName}
      onclick={() => onOpen(tab.name)}
    >
      {tab.label}
      <span class="count">{tab.count}</span>
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius-control);
    background: var(--color-fill);
  }

  .tab {
    height: 28px;
    padding: 0 var(--space-3);
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

  .tab:hover {
    color: var(--color-text);
  }

  .tab.is-open {
    background: var(--color-card);
    color: var(--color-text);
    font-weight: 600;
    box-shadow: var(--shadow-raised);
  }

  .count {
    margin-left: var(--space-1);
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }
</style>
