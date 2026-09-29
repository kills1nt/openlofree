<script lang="ts">
  import { t } from '../lib/strings';
  import type { AppError } from '../lib/types';

  interface Props {
    kind: 'searching' | 'loading' | 'error';
    error?: AppError | null;
    layers?: number;
    onretry?: () => void;
    ondemo?: () => void;
  }

  let { kind, error = null, layers = 0, onretry, ondemo }: Props = $props();

  const copy = $derived.by(() => {
    if (kind === 'loading') return { glyph: '…', title: t.state.loading.title, body: t.state.loading.body(layers), hint: '' };
    if (kind === 'searching' && !error) return { glyph: '…', title: t.chip.searching, body: '', hint: '' };
    if (error?.kind === 'timeout') return { glyph: 'z', ...t.state.timeout };
    if (kind === 'searching') return { glyph: '?', ...t.state.notFound };
    return { glyph: '!', title: t.state.generic.title, body: error?.message ?? '', hint: '' };
  });
  const busy = $derived(kind === 'loading' || (kind === 'searching' && !error));
</script>

<section class="state" role={busy ? 'status' : 'alert'} aria-live="polite">
  <div class="glyph" class:busy aria-hidden="true"><span>{copy.glyph}</span></div>
  <h1>{copy.title}</h1>
  {#if copy.body}<p class="body">{copy.body}</p>{/if}
  {#if busy}
    <div class="bar" aria-hidden="true"><span></span></div>
  {:else}
    <div class="actions">
      <button class="btn primary" onclick={onretry}>{t.state.retry}</button>
      <button class="btn" onclick={ondemo}>{t.state.demo}</button>
    </div>
    <p class="hint">{t.state.demoNote}</p>
    {#if copy.hint}<p class="hint">{copy.hint}</p>{/if}
  {/if}
</section>

<style>
  .state {
    display: grid;
    justify-items: center;
    gap: 14px;
    text-align: center;
    max-width: 520px;
    margin: 8vh auto 0;
    padding: 0 8px;
  }

  .glyph {
    display: grid;
    place-items: center;
    width: 96px;
    height: 96px;
    background: var(--key);
    border-radius: 22px;
    box-shadow: 0 6px 0 var(--key-edge);
    font-size: 2.6rem;
    font-weight: 650;
    margin-bottom: 8px;
  }

  .glyph.busy span {
    animation: nudge 1.2s ease-in-out infinite;
  }

  .body {
    max-width: 46ch;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 12px;
    margin-top: 6px;
  }

  .bar {
    width: 220px;
    height: 6px;
    border-radius: 3px;
    background: var(--line);
    overflow: hidden;
  }

  .bar span {
    display: block;
    width: 40%;
    height: 100%;
    background: var(--ink);
    animation: slide 1.1s ease-in-out infinite;
  }

  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(260%);
    }
  }

  @keyframes nudge {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(4px);
    }
  }
</style>
