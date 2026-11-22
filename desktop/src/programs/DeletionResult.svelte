<script lang="ts">
  import type { DeletionReport } from "./program-api";
  import { describeLocation, summarizeDeletion } from "./program-wording";

  interface Props {
    report: DeletionReport;
    onDismiss: () => void;
  }

  let { report, onDismiss }: Props = $props();

  const failures = $derived(report.outcomes.filter((outcome) => outcome.status === "failed"));
</script>

<div class="result" class:has-failures={failures.length > 0} role="status">
  <div class="account">
    <p class="summary">{summarizeDeletion(report)}</p>
    {#each failures as failure (describeLocation(failure.location))}
      <p class="detail">{describeLocation(failure.location)}: {failure.reason}</p>
    {/each}
    {#if report.registryBackupDirectory !== null}
      <p class="detail">Registry backup saved in {report.registryBackupDirectory}</p>
    {/if}
  </div>
  <button class="button" onclick={onDismiss}>Dismiss</button>
</div>

<style>
  .result {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin: var(--space-4) var(--space-6) 0;
    padding: var(--space-3) var(--space-3) var(--space-3) var(--space-4);
    border: 1px solid var(--color-stroke);
    border-radius: var(--radius-surface);
    background: var(--color-card);
    animation: arrive var(--duration-normal) var(--ease-enter);
  }

  @keyframes arrive {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
  }

  .result.has-failures {
    background: var(--color-destructive-fill);
  }

  .account {
    flex: 1;
    min-width: 0;
  }

  .summary {
    margin: 0;
    font: var(--text-body-strong);
    color: var(--color-created);
  }

  .result.has-failures .summary {
    color: var(--color-destructive);
  }

  .detail {
    margin: var(--space-1) 0 0;
    font: var(--text-caption);
    color: var(--color-text-secondary);
    overflow-wrap: anywhere;
    user-select: text;
  }
</style>
