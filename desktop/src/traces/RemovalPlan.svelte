<script lang="ts">
  import type { RemovalStep } from "./trace-api";
  import { removalActionLabel } from "./trace-wording";

  interface Props {
    steps: readonly RemovalStep[];
  }

  let { steps }: Props = $props();
</script>

{#if steps.length === 0}
  <p class="introduction">This program left nothing behind, so there is nothing to remove.</p>
{:else}
  <p class="introduction">
    These steps, in this order, undo everything the program changed. Entries marked Review existed
    before the program ran, and their original content is not known yet. Running the plan from the
    app arrives in version 0.2.
  </p>
  <ol class="steps">
    {#each steps as step, position (position)}
      <li class="step">
        <span class="action action-{step.action}">{removalActionLabel(step.action)}</span>
        <span class="target" title={step.target}>{step.target}</span>
      </li>
    {/each}
  </ol>
{/if}

<style>
  .introduction {
    max-width: 640px;
    margin: var(--space-3) var(--space-2) var(--space-4);
    color: var(--color-text-secondary);
  }

  .steps {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .step {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    height: var(--row-height);
    padding: 0 var(--space-2);
    border-radius: var(--radius-control);
  }

  .step:hover {
    background: var(--color-hover);
  }

  .action {
    flex: none;
    width: 56px;
    font: var(--text-caption);
  }

  .action-delete {
    color: var(--color-deleted);
  }

  .action-restore {
    color: var(--color-created);
  }

  .action-review {
    color: var(--color-changed);
  }

  .target {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
