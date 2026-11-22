<script lang="ts" module>
  export interface Fact {
    readonly label: string;
    readonly value: string;
  }
</script>

<script lang="ts">
  interface Props {
    facts: readonly Fact[];
  }

  let { facts }: Props = $props();
</script>

<dl class="facts">
  {#each facts as fact (fact.label)}
    <div class="fact">
      <dt class="label">{fact.label}</dt>
      <dd class="value">{fact.value}</dd>
    </div>
  {/each}
</dl>

<style>
  .facts {
    display: flex;
    margin: var(--space-6) 0 0;
  }

  .fact {
    padding: 0 var(--space-6);
    border-left: 1px solid var(--color-divider);
  }

  .fact:first-child {
    padding-left: 0;
    border-left: 0;
  }

  .label {
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }

  .value {
    margin: 2px 0 0;
    font: var(--text-headline);
    font-variant-numeric: tabular-nums;
  }
</style>
