<script lang="ts">
  import { app, type Tab } from '../lib/app.svelte';
  import { t } from '../lib/strings';
  import { theme } from '../lib/theme.svelte';

  const tabs: Tab[] = ['keys', 'lighting', 'profiles', 'device'];

  function onkey(e: KeyboardEvent, i: number) {
    const to = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : -1;
    if (to < 0) return;
    e.preventDefault();
    const next = tabs[(to + tabs.length) % tabs.length]!;
    app.tab = next;
    document.getElementById(`tab-${next}`)?.focus();
  }

  const chip = $derived(
    app.phase === 'ready' && app.info
      ? { text: app.info.demo ? t.chip.demo : app.info.label, state: app.info.demo ? 'demo' : 'on' }
      : { text: app.phase === 'searching' && !app.error ? t.chip.searching : t.chip.none, state: 'off' },
  );
  const themeText = $derived(t.theme[theme.current]);
</script>

<header class="bar">
  <a class="skip" href="#main">Skip to content</a>
  <div class="brand"><span class="mark" aria-hidden="true"></span>{t.name}</div>

  <div class="tabs" role="tablist" aria-label="Sections">
    {#each tabs as id, i (id)}
      <button
        role="tab"
        id="tab-{id}"
        aria-selected={app.tab === id}
        aria-controls="panel-{id}"
        tabindex={app.tab === id ? 0 : -1}
        onclick={() => (app.tab = id)}
        onkeydown={(e) => onkey(e, i)}>{t.tabs[id]}</button
      >
    {/each}
  </div>

  <div class="right">
    <div class="chip" data-state={chip.state}>
      <span class="dot" aria-hidden="true"></span>{chip.text}
    </div>
    <button class="btn small" onclick={() => theme.cycle()} title={t.theme.label}>{t.theme.label}: {themeText}</button>
  </div>
</header>

<style>
  .bar {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 24px;
    padding: 12px 28px;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
  }

  .skip {
    position: absolute;
    left: -9999px;
    background: var(--surface);
    padding: 8px 12px;
    border-radius: var(--r-input);
  }

  .skip:focus {
    left: 12px;
    top: 8px;
    z-index: 10;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 700;
    letter-spacing: -0.01em;
    font-size: 1.1rem;
  }

  .mark {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: var(--key);
    box-shadow:
      0 2px 0 var(--key-edge),
      inset 0 0 0 5px var(--key);
    position: relative;
  }

  .mark::after {
    content: '';
    position: absolute;
    inset: 6px;
    border-radius: 2px;
    background: var(--accent);
  }

  .tabs {
    display: flex;
    gap: 4px;
  }

  .tabs button {
    font: inherit;
    font-weight: 560;
    color: var(--muted);
    background: none;
    border: 0;
    border-bottom: 3px solid transparent;
    padding: 8px 14px;
    cursor: pointer;
    transition: color 120ms ease;
  }

  .tabs button:hover {
    color: var(--ink);
  }

  .tabs button[aria-selected='true'] {
    color: var(--ink);
    border-bottom-color: var(--ink);
  }

  .right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border: 1px solid var(--control-edge);
    border-radius: var(--r-input);
    font-weight: 560;
    white-space: nowrap;
  }

  .dot {
    width: 9px;
    height: 9px;
    border-radius: 2px;
    background: var(--muted);
  }

  .chip[data-state='on'] .dot {
    background: var(--ok);
  }

  .chip[data-state='demo'] .dot {
    background: var(--accent);
    box-shadow: 0 0 0 1.5px var(--ink);
  }

  @media (max-width: 900px) {
    .bar {
      grid-template-columns: 1fr;
      gap: 10px;
      padding: 10px 16px;
    }

    .right {
      flex-wrap: wrap;
    }
  }
</style>
