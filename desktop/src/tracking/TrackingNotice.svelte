<script lang="ts">
  import ShellIcon from "../shell/ShellIcon.svelte";

  const ICON_SIZE = 64;
  const PATH_SEPARATOR = "\\";

  interface Props {
    programPath: string;
    onStop: () => void;
  }

  let { programPath, onStop }: Props = $props();

  const programName = $derived(programPath.slice(programPath.lastIndexOf(PATH_SEPARATOR) + 1));
</script>

<section class="tracking" aria-live="polite">
  <ShellIcon path={programPath} size={ICON_SIZE} placeholder="program" />
  <h1 class="title">Watching {programName}</h1>
  <p class="explanation">
    Finish its installer or use it as usual. Nazar stops by itself once the program and everything it
    started have closed.
  </p>
  <div class="activity" role="progressbar" aria-label="Watching the program"></div>
  <button class="button" onclick={onStop}>Stop watching</button>
</section>

<style>
  .tracking {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    text-align: center;
  }

  .title {
    margin: var(--space-5) 0 var(--space-2);
    font: var(--text-title);
    letter-spacing: var(--tracking-title);
  }

  .explanation {
    max-width: 440px;
    margin: 0;
    color: var(--color-text-secondary);
  }

  .activity {
    position: relative;
    width: 200px;
    height: 4px;
    margin: var(--space-6) 0;
    overflow: hidden;
    border-radius: var(--radius-pill);
    background: var(--color-fill);
  }

  .activity::after {
    content: "";
    position: absolute;
    inset: 0 60% 0 0;
    border-radius: var(--radius-pill);
    background: var(--color-accent-text);
    animation: travel 1.6s ease-in-out infinite;
  }

  @keyframes travel {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }
</style>
