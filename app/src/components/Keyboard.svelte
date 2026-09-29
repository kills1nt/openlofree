<script lang="ts">
  import { untrack } from 'svelte';
  import { keycodeForEvent } from '../lib/eventcodes';
  import { isChanged, keyIndex } from '../lib/draft';
  import { bounds, keyId, neighbor, type Dir } from '../lib/geometry';
  import { t } from '../lib/strings';
  import type { KeyDef, Layers } from '../lib/types';

  interface Props {
    keys: KeyDef[];
    cols: number;
    layer: number;
    draft: Layers;
    /** What the keyboard holds now. Keys that differ from `draft` get a changed mark. */
    saved?: Layers;
    legend: (code: number) => string;
    selected?: string | null;
    /** Makes the keys buttons that select. Without it the keyboard is a picture. */
    onselect?: (id: string) => void;
    /** Lower the drawn key when the same physical key is held. */
    press?: boolean;
    /** Backlight preview: 0 to 1 brightness, drawn on a dark stage. */
    light?: number;
    breathing?: boolean;
    label?: string;
  }

  let { keys, cols, layer, draft, saved, legend, selected = null, onselect, press = false, light, breathing = false, label = 'Keyboard' }: Props = $props();

  const size = $derived(bounds(keys));
  const interactive = $derived(!!onselect);
  let pressed = $state<Set<string>>(new Set());
  let focusId = $state<string | null>(null);
  let root: HTMLDivElement | undefined = $state();

  const rovingId = $derived(selected ?? focusId ?? (keys[0] ? keyId(keys[0]) : null));

  const codeOf = (k: KeyDef) => draft[layer]?.[keyIndex(cols, k.row, k.col)] ?? 0;

  function move(k: KeyDef, dir: Dir) {
    const next = neighbor(keys, k, dir);
    if (!next) return;
    const id = keyId(next);
    focusId = id;
    root?.querySelector<HTMLElement>(`[data-id="${id}"]`)?.focus();
  }

  function onkey(e: KeyboardEvent, k: KeyDef) {
    const dir: Dir | undefined = ({ ArrowLeft: 'left', ArrowRight: 'right', ArrowUp: 'up', ArrowDown: 'down' } as const)[
      e.key as 'ArrowLeft'
    ];
    if (dir) {
      e.preventDefault();
      move(k, dir);
    }
  }

  const typing = (t: EventTarget | null) =>
    t instanceof HTMLElement && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT');

  function down(e: KeyboardEvent) {
    if (!press || e.repeat || typing(e.target)) return;
    const code = keycodeForEvent(e.code);
    if (code === undefined) return;
    const hit = keys.filter((k) => codeOf(k) === code).map(keyId);
    if (hit.length) pressed = new Set([...pressed, ...hit]);
  }

  function up(e: KeyboardEvent) {
    if (!press || pressed.size === 0) return;
    const code = keycodeForEvent(e.code);
    if (code === undefined) return;
    const release = new Set(keys.filter((k) => codeOf(k) === code).map(keyId));
    pressed = new Set([...pressed].filter((id) => !release.has(id)));
  }

  $effect(() => {
    // A layer change while a key is held must not leave a stale pressed key behind.
    layer;
    untrack(() => (pressed = new Set()));
  });
</script>

<svelte:window onkeydown={down} onkeyup={up} onblur={() => (pressed = new Set())} />

<div class="wrap">
  <div
    class="kb"
    class:night={light !== undefined}
    class:breathing
    bind:this={root}
    role={interactive ? 'group' : 'img'}
    aria-label={interactive ? `${label}, ${t.keys.layer(layer)}` : label}
    style="--cols:{size.w}; --rows:{size.h}; --light:{light ?? 1}"
  >
    {#each keys as k (keyId(k))}
      {@const id = keyId(k)}
      {@const code = codeOf(k)}
      {@const text = legend(code)}
      {@const changed = saved ? isChanged(saved, draft, layer, keyIndex(cols, k.row, k.col)) : false}
      {@const style = `--x:${k.x}; --y:${k.y}; --w:${k.w}; --h:${k.h}; --d:${k.col * 9}ms`}
      {#if interactive}
        <button
          class="key"
          class:selected={selected === id}
          class:pressed={pressed.has(id)}
          class:changed
          data-id={id}
          {style}
          tabindex={rovingId === id ? 0 : -1}
          aria-pressed={selected === id}
          aria-label={t.keys.keyLabel(k.row, k.col, text, changed)}
          onclick={() => onselect?.(id)}
          onfocus={() => (focusId = id)}
          onkeydown={(e) => onkey(e, k)}
        >
          <span class="cap">
            {#key layer}<span class="legend" class:long={text.length > 5 && k.w < 1.5} class:dim={code <= 1}>{text}</span>{/key}
            {#if changed}<span class="mark" aria-hidden="true"></span>{/if}
          </span>
        </button>
      {:else}
        <div class="key" class:pressed={pressed.has(id)} {style} aria-hidden="true">
          <span class="cap">
            {#key layer}<span class="legend" class:long={text.length > 5 && k.w < 1.5} class:dim={code <= 1}>{text}</span>{/key}
          </span>
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .wrap {
    container-type: inline-size;
    width: 100%;
    max-width: 1120px;
    margin: 0 auto;
    /* Narrow windows scroll the keyboard sideways instead of shrinking keys below a usable size. */
    overflow-x: auto;
    padding-bottom: 4px;
  }

  .kb {
    --u: max(calc(100cqw / var(--cols)), 42px);
    position: relative;
    width: calc(var(--cols) * var(--u));
    height: calc(var(--rows) * var(--u) + var(--u) * 0.12);
  }

  .key {
    position: absolute;
    left: calc(var(--x) * var(--u));
    top: calc(var(--y) * var(--u));
    width: calc(var(--w) * var(--u));
    height: calc(var(--h) * var(--u));
    padding: calc(var(--u) * 0.045);
    margin: 0;
    border: 0;
    background: none;
    font: inherit;
    color: inherit;
    cursor: default;
  }

  button.key {
    cursor: pointer;
  }

  .cap {
    position: relative;
    display: grid;
    place-items: center;
    width: 100%;
    height: 100%;
    background: var(--key);
    border-radius: clamp(4px, calc(var(--u) * 0.14), var(--r-key));
    box-shadow: 0 calc(var(--u) * 0.06) 0 var(--key-edge);
    transform: translateY(0);
    transition:
      transform 120ms ease,
      box-shadow 120ms ease;
  }

  button.key:hover .cap {
    transform: translateY(-1px);
  }

  .key.pressed .cap,
  button.key:active .cap {
    transform: translateY(calc(var(--u) * 0.05));
    box-shadow: 0 calc(var(--u) * 0.015) 0 var(--key-edge);
  }

  .key.selected .cap {
    background: var(--accent);
    color: var(--on-accent);
    box-shadow:
      0 calc(var(--u) * 0.06) 0 var(--accent-edge),
      0 0 0 2px var(--ink);
    transform: translateY(-2px);
  }

  .key.selected.pressed .cap {
    transform: translateY(calc(var(--u) * 0.04));
  }

  button.key:focus-visible {
    outline: none;
  }

  button.key:focus-visible .cap {
    outline: 3px solid var(--focus);
    outline-offset: 2px;
  }

  .legend {
    max-width: 100%;
    padding: 0 3%;
    overflow: hidden;
    text-align: center;
    line-height: 1.05;
    overflow-wrap: break-word;
    font-size: clamp(9px, calc(var(--u) * 0.27), 17px);
    font-weight: 560;
    animation: legend-in 220ms ease-out both;
    animation-delay: var(--d);
  }

  /* None and transparent keys recede so real assignments stand out. */
  .legend.dim {
    color: var(--muted);
    font-weight: 400;
  }

  .night .legend.dim {
    color: color-mix(in srgb, var(--lit) calc(var(--light) * 55%), var(--unlit));
  }

  .legend.long {
    font-size: clamp(7.5px, calc(var(--u) * 0.175), 12px);
  }

  .mark {
    position: absolute;
    top: 5%;
    right: 5%;
    width: clamp(5px, calc(var(--u) * 0.11), 8px);
    height: clamp(5px, calc(var(--u) * 0.11), 8px);
    border-radius: 2px;
    background: var(--ink);
  }

  @keyframes legend-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Backlight preview: caps stay dark and the legends carry the light, like the real keyboard at night. */
  .night {
    --key: #2a241d;
    --key-edge: #0f0d0a;
    --lit: #fff1d2;
    --unlit: #7a705f;
  }

  .night .legend {
    color: color-mix(in srgb, var(--lit) calc(var(--light) * 100%), var(--unlit));
  }

  .breathing .legend {
    animation:
      legend-in 220ms ease-out both,
      breathe 3.2s ease-in-out 220ms infinite;
    animation-delay: var(--d), 0ms;
  }

  @keyframes breathe {
    0%,
    100% {
      opacity: 0.35;
    }
    50% {
      opacity: 1;
    }
  }
</style>
