<script lang="ts">
  import type { Snippet } from "svelte";

  import ShellIcon from "../shell/ShellIcon.svelte";

  const ICON_SIZE = 64;

  interface Props {
    iconSource: string | null;
    title: string;
    subtitle: string;
    children?: Snippet | undefined;
  }

  let { iconSource, title, subtitle, children }: Props = $props();
</script>

<div class="identity">
  <ShellIcon path={iconSource} size={ICON_SIZE} placeholder="program" />
  <div class="names">
    <h1 class="title" {title}>{title}</h1>
    <p class="subtitle" title={subtitle}>{subtitle}</p>
  </div>
  {@render children?.()}
</div>

<style>
  .identity {
    display: flex;
    align-items: center;
    gap: var(--space-5);
  }

  .names {
    flex: 1;
    min-width: 0;
  }

  .title,
  .subtitle {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    user-select: text;
  }

  .title {
    font: var(--text-title);
    letter-spacing: var(--tracking-title);
  }

  .subtitle {
    margin-top: var(--space-1);
    font: var(--text-headline);
    font-weight: 400;
    color: var(--color-text-secondary);
  }
</style>
