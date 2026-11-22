<script lang="ts">
  interface Props {
    message: string;
    onDismiss: () => void;
  }

  let { message, onDismiss }: Props = $props();
</script>

<div class="notice" role="alert">
  <p class="message">{message}</p>
  <button class="button" onclick={onDismiss}>Dismiss</button>
</div>

<style>
  .notice {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin: var(--space-4) var(--space-6) 0;
    padding: var(--space-3) var(--space-3) var(--space-3) var(--space-4);
    border-radius: var(--radius-surface);
    background: var(--color-destructive-fill);
    animation: arrive var(--duration-normal) var(--ease-enter);
  }

  @keyframes arrive {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
  }

  .message {
    flex: 1;
    margin: 0;
    color: var(--color-destructive);
    user-select: text;
  }
</style>
