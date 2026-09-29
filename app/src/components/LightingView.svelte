<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/strings';
  import type { Mode } from '../lib/types';
  import Keyboard from './Keyboard.svelte';

  const info = $derived(app.info!);
  const b = $derived(app.draftBacklight);
  const busy = $derived(app.lightBusy);

  const modes: Mode[] = ['off', 'steady', 'breathing'];
  const percent = $derived(Math.round((b.brightness / 255) * 100));
  const level = $derived(b.mode === 'off' ? 0 : b.brightness / 255);

  // The device reports brightness 0 while off, so remember the last lit value to come back to.
  let lastOn = $state(255);
  $effect(() => {
    if (b.brightness > 0) lastOn = b.brightness;
  });

  function setMode(mode: Mode) {
    app.previewBacklight({ mode, brightness: mode === 'off' ? b.brightness : b.brightness || lastOn });
  }

  function setPercent(e: Event) {
    const p = Number((e.currentTarget as HTMLInputElement).value);
    const raw = Math.round((p / 100) * 255);
    app.previewBacklight({ mode: b.mode === 'off' && p > 0 ? 'steady' : b.mode, brightness: raw });
  }

  function modeKey(e: KeyboardEvent, i: number) {
    const to = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : -1;
    if (to < 0 || to >= modes.length) return;
    e.preventDefault();
    setMode(modes[to]!);
    document.getElementById(`mode-${modes[to]}`)?.focus();
  }
</script>

<section class="light">
  <h1>{t.lighting.title}</h1>

  <div class="stage panel night-panel">
    <Keyboard
      keys={info.keys}
      cols={info.cols}
      layer={0}
      draft={app.draftLayers}
      legend={(c) => app.legend(c)}
      light={level}
      breathing={b.mode === 'breathing'}
      label="Backlight preview"
    />
  </div>

  <div class="controls panel">
    <div class="row">
      <span id="mode-label" class="lbl">{t.lighting.mode}</span>
      <div class="segmented" role="radiogroup" aria-labelledby="mode-label">
        {#each modes as m, i (m)}
          <button
            role="radio"
            id="mode-{m}"
            aria-checked={b.mode === m}
            tabindex={b.mode === m ? 0 : -1}
            onclick={() => setMode(m)}
            onkeydown={(e) => modeKey(e, i)}>{t.lighting.modes[m]}</button
          >
        {/each}
      </div>
    </div>

    <div class="row">
      <label class="lbl" for="brightness">{t.lighting.brightness}</label>
      <div class="slider">
        <input id="brightness" type="range" min="0" max="100" step="1" value={b.mode === 'off' ? 0 : percent} oninput={setPercent} />
        <output for="brightness">{b.mode === 'off' ? 0 : percent}%</output>
      </div>
    </div>

    <p class="hint">{t.lighting.note}</p>
    <p class="hint">{t.lighting.whiteOnly}</p>

    <div class="actions">
      <p class="status" class:error-text={busy.kind === 'error'} class:ok-text={busy.kind === 'done'} role={busy.kind === 'error' ? 'alert' : undefined}>
        {#if busy.kind === 'writing'}{t.bar.writing}
        {:else if busy.kind === 'error'}{busy.message}
        {:else if busy.kind === 'done'}{busy.message}{/if}
      </p>
      <button class="btn" onclick={() => app.resetBacklight()} disabled={!app.lightChanged}>{t.lighting.reset}</button>
      <button class="btn primary" onclick={() => app.applyBacklight()} disabled={!app.lightChanged || busy.kind === 'writing'}>{t.lighting.apply}</button>
    </div>
  </div>
</section>

<style>
  .light {
    display: grid;
    gap: 20px;
  }

  .night-panel {
    background: #15120e;
    border-color: #38312a;
    padding: 24px;
  }

  .controls {
    display: grid;
    gap: 18px;
  }

  .row {
    display: grid;
    grid-template-columns: 140px 1fr;
    align-items: center;
    gap: 12px;
  }

  .lbl {
    font-weight: 650;
  }

  .slider {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  input[type='range'] {
    flex: 1;
    max-width: 420px;
    accent-color: var(--ink);
    min-height: 32px;
  }

  output {
    min-width: 3.5ch;
    font-weight: 650;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: flex-end;
  }

  .status {
    margin-right: auto;
    font-weight: 560;
  }

  @media (max-width: 700px) {
    .row {
      grid-template-columns: 1fr;
    }

    .actions {
      flex-wrap: wrap;
    }
  }
</style>
