<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/strings';
  import Keyboard from './Keyboard.svelte';
  import Picker from './Picker.svelte';

  const info = $derived(app.info!);
  const busy = $derived(app.keysBusy);
  const changed = $derived(app.changes.keys > 0);

  function layerKey(e: KeyboardEvent, i: number) {
    const to = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : -1;
    if (to < 0 || to >= info.layers) return;
    e.preventDefault();
    app.setLayer(to);
    document.getElementById(`layer-${to}`)?.focus();
  }
</script>

<section class="keys">
  <header class="head">
    <h1>{t.tabs.keys}</h1>
    <div class="layers" role="tablist" aria-label={t.keys.layers}>
      {#each { length: info.layers } as _, i (i)}
        <button
          role="tab"
          id="layer-{i}"
          aria-selected={app.layer === i}
          tabindex={app.layer === i ? 0 : -1}
          onclick={() => app.setLayer(i)}
          onkeydown={(e) => layerKey(e, i)}>{i}<span class="sr-only"> {t.keys.layer(i)}</span></button
        >
      {/each}
    </div>
  </header>

  <div class="stage panel">
    <Keyboard
      keys={info.keys}
      cols={info.cols}
      layer={app.layer}
      draft={app.draftLayers}
      saved={app.saved?.layers}
      legend={(c) => app.legend(c)}
      selected={app.selected}
      onselect={(id) => app.selectKey(id)}
      press
      label={info.label}
    />
  </div>

  <Picker />

  <div class="apply" role="group" aria-label="Apply changes">
    <p class="status" class:error-text={busy.kind === 'error'} class:ok-text={busy.kind === 'done'} role={busy.kind === 'error' ? 'alert' : undefined}>
      {#if busy.kind === 'writing'}{t.bar.writing}
      {:else if busy.kind === 'error'}{t.bar.failed}. {busy.message}
      {:else if busy.kind === 'done' && !changed}{busy.message}
      {:else if changed}{t.bar.summary(app.changes.keys, app.changes.layers)}
      {:else}<span class="muted">{t.bar.none}</span>{/if}
    </p>
    <button class="btn" onclick={() => app.discard()} disabled={!changed || busy.kind === 'writing'}>{t.bar.discard}</button>
    <button class="btn primary" onclick={() => app.applyKeymap()} disabled={!changed || busy.kind === 'writing'}>{t.bar.apply}</button>
  </div>
</section>

<style>
  .keys {
    display: grid;
    gap: 20px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .layers {
    display: flex;
    gap: 8px;
  }

  .layers button {
    font: inherit;
    font-weight: 650;
    min-width: 44px;
    min-height: 40px;
    color: var(--ink);
    background: var(--key);
    border: 1px solid var(--control-edge);
    border-radius: var(--r-button);
    cursor: pointer;
    box-shadow: 0 3px 0 var(--key-edge);
    transition:
      transform 120ms ease,
      box-shadow 120ms ease;
  }

  .layers button[aria-selected='true'] {
    background: var(--accent);
    color: var(--on-accent);
    border-color: var(--ink);
    box-shadow: 0 1px 0 var(--accent-edge);
    transform: translateY(2px);
  }

  .stage {
    padding: 22px;
  }

  .apply {
    position: sticky;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    padding: 14px 0 16px;
    background: var(--bg);
    border-top: 1px solid var(--line);
  }

  .status {
    margin-right: auto;
    font-weight: 560;
  }

  @media (max-width: 700px) {
    .apply {
      flex-wrap: wrap;
    }

    .status {
      flex-basis: 100%;
    }
  }
</style>
