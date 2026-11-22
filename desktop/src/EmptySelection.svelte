<script lang="ts">
  const MARK_SIZE = 88;

  interface Props {
    title: string;
    explanation: string;
    actionLabel?: string | undefined;
    actionNote?: string | undefined;
    onAction?: (() => void) | undefined;
  }

  let { title, explanation, actionLabel, actionNote, onAction }: Props = $props();
</script>

<section class="empty-selection">
  <img class="mark" src="/nazar-mark.png" alt="" width={MARK_SIZE} height={MARK_SIZE} />
  <h1 class="title">{title}</h1>
  <p class="explanation">{explanation}</p>
  {#if actionLabel !== undefined && onAction !== undefined}
    <button class="button button-primary action" onclick={onAction}>{actionLabel}</button>
  {/if}
  {#if actionNote !== undefined}
    <p class="note">{actionNote}</p>
  {/if}
</section>

<style>
  .empty-selection {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    text-align: center;
  }

  .mark {
    margin-bottom: var(--space-5);
  }

  .title {
    margin: 0 0 var(--space-2);
    font: var(--text-title);
    letter-spacing: var(--tracking-title);
  }

  .explanation {
    max-width: 440px;
    margin: 0;
    color: var(--color-text-secondary);
  }

  .action {
    margin-top: var(--space-6);
  }

  .note {
    max-width: 440px;
    margin: var(--space-3) 0 0;
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }
</style>
