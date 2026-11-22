<script lang="ts">
  interface Props {
    title: string;
    explanation: string;
    caution?: string | undefined;
    items?: readonly string[] | undefined;
    confirmLabel: string;
    isDestructive: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    explanation,
    caution,
    items,
    confirmLabel,
    isDestructive,
    onConfirm,
    onCancel,
  }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    dialog?.showModal();
  });
</script>

<dialog class="dialog" bind:this={dialog} oncancel={onCancel}>
  <h2 class="title">{title}</h2>
  <p class="explanation">{explanation}</p>
  {#if caution !== undefined}
    <p class="caution">{caution}</p>
  {/if}
  {#if items !== undefined && items.length > 0}
    <ul class="items">
      {#each items as item (item)}
        <li class="item">{item}</li>
      {/each}
    </ul>
  {/if}
  <div class="actions">
    <button class="button" onclick={onCancel}>Cancel</button>
    <button
      class="button"
      class:button-destructive={isDestructive}
      class:button-primary={!isDestructive}
      onclick={onConfirm}
    >
      {confirmLabel}
    </button>
  </div>
</dialog>

<style>
  .dialog {
    width: 460px;
    max-width: calc(100vw - var(--space-8));
    padding: var(--space-6);
    border: 1px solid var(--color-stroke);
    border-radius: var(--radius-surface);
    background: var(--color-window);
    color: var(--color-text);
    box-shadow: var(--shadow-dialog);
    animation: rise var(--duration-normal) var(--ease-enter);
  }

  .dialog::backdrop {
    background: var(--color-backdrop);
    animation: fade var(--duration-fast) linear;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  .title {
    margin: 0 0 var(--space-2);
    font: var(--text-subtitle);
    letter-spacing: var(--tracking-title);
  }

  .explanation,
  .caution {
    margin: 0 0 var(--space-3);
    color: var(--color-text-secondary);
  }

  .caution {
    color: var(--color-changed);
  }

  .items {
    max-height: 168px;
    margin: 0 0 var(--space-3);
    padding: var(--space-2) var(--space-3);
    overflow-y: auto;
    border-radius: var(--radius-control);
    background: var(--color-fill);
    list-style: none;
  }

  .item {
    font: var(--text-caption);
    overflow-wrap: anywhere;
  }

  .item + .item {
    margin-top: var(--space-1);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-5);
  }
</style>
