<script lang="ts">
  import ShellIcon from "../shell/ShellIcon.svelte";

  const ICON_SIZE = 32;

  interface Props {
    title: string;
    details: string;
    trailingDetail: string;
    iconSource?: string | null | undefined;
    isOpen: boolean;
    onOpen: () => void;
  }

  let { title, details, trailingDetail, iconSource, isOpen, onOpen }: Props = $props();
</script>

<li>
  <button
    class="item"
    class:is-open={isOpen}
    aria-current={isOpen ? "page" : undefined}
    onclick={onOpen}
  >
    {#if iconSource !== undefined}
      <ShellIcon path={iconSource} size={ICON_SIZE} placeholder="program" />
    {/if}
    <span class="texts">
      <span class="title" {title}>{title}</span>
      <span class="details">{details}</span>
    </span>
    <span class="trailing">{trailingDetail}</span>
  </button>
</li>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    height: var(--navigation-item-height);
    padding: 0 var(--space-3);
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    cursor: default;
    transition: background var(--duration-faster) linear;
  }

  .item:hover {
    background: var(--color-hover);
  }

  .item.is-open {
    background: var(--color-selected);
  }

  .texts {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .title,
  .details {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title {
    font: var(--text-body);
    font-weight: 500;
  }

  .item.is-open .title {
    font-weight: 600;
  }

  .details,
  .trailing {
    font: var(--text-caption);
    color: var(--color-text-secondary);
  }

  .trailing {
    flex: none;
    font-variant-numeric: tabular-nums;
  }
</style>
