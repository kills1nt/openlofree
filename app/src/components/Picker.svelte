<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/strings';

  let query = $state('');
  let custom = $state('');
  let customError = $state(false);

  const hex = (c: number) => `0x${c.toString(16).toUpperCase().padStart(4, '0')}`;
  const q = $derived(query.trim().toLowerCase());
  const groups = $derived(
    app.catalog
      .map((g) => ({
        name: g.name,
        items: q
          ? g.items.filter(
              (i) => i.label.toLowerCase().includes(q) || hex(i.code).toLowerCase().includes(q) || g.name.toLowerCase().includes(q),
            )
          : g.items,
      }))
      .filter((g) => g.items.length > 0),
  );
  const key = $derived(app.selectedKey);
  const code = $derived(app.selectedCode);

  function setCustom(e: Event) {
    e.preventDefault();
    const v = custom.trim();
    if (!/^(0x)?[0-9a-f]{1,4}$/i.test(v)) {
      customError = true;
      return;
    }
    customError = false;
    void app.assign(Number.parseInt(v.replace(/^0x/i, ''), 16));
    custom = '';
  }
</script>

<section class="assign" aria-labelledby="assign-title">
  <div class="now panel">
    <h2 id="assign-title">{t.picker.title}</h2>
    {#if key && code !== null}
      <p class="hint">{t.keys.layer(app.layer)}, row {key.row + 1}, column {key.col + 1}</p>
      <div class="preview" aria-live="polite">
        <div class="cap"><span>{app.legend(code)}</span></div>
        <div>
          <div class="label">{t.keys.now}: {app.legend(code)}</div>
          <div class="hint">{hex(code)}</div>
        </div>
      </div>
      <form onsubmit={setCustom} class="custom">
        <label class="field">
          {t.picker.customCode}
          <input
            class="input"
            bind:value={custom}
            placeholder="0x0029"
            autocomplete="off"
            spellcheck="false"
            aria-invalid={customError}
            aria-describedby="custom-hint"
          />
        </label>
        <button class="btn" type="submit" disabled={custom.trim() === ''}>{t.picker.set}</button>
      </form>
      <p id="custom-hint" class="hint" class:error-text={customError}>
        {customError ? t.picker.invalid : t.picker.customHint}
      </p>
    {:else}
      <p class="prompt">{t.keys.selectPrompt}</p>
    {/if}
  </div>

  <div class="list panel">
    <label class="field">
      <span class="sr-only">{t.picker.search}</span>
      <input class="input" type="search" placeholder={t.picker.search} bind:value={query} autocomplete="off" />
    </label>
    <div class="groups" tabindex="-1">
      {#each groups as g (g.name)}
        <div class="group">
          <h3>{g.name}</h3>
          <div class="items">
            {#each g.items as item (item.code)}
              <button
                class="item"
                class:current={item.code === code}
                disabled={!key}
                aria-pressed={item.code === code}
                title={hex(item.code)}
                onclick={() => app.assign(item.code)}>{item.label}</button
              >
            {/each}
          </div>
        </div>
      {:else}
        <p class="hint">{t.picker.noMatch(query)}</p>
      {/each}
    </div>
  </div>
</section>

<style>
  .assign {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 20px;
    align-items: start;
  }

  .now {
    display: grid;
    gap: 12px;
  }

  .prompt {
    color: var(--muted);
  }

  .preview {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .cap {
    display: grid;
    place-items: center;
    min-width: 72px;
    height: 64px;
    padding: 0 10px;
    background: var(--accent);
    color: var(--on-accent);
    border-radius: 12px;
    box-shadow:
      0 4px 0 var(--accent-edge),
      0 0 0 2px var(--ink);
    font-weight: 650;
    font-size: 1.1rem;
  }

  .label {
    font-weight: 650;
  }

  .custom {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
    align-items: end;
  }

  .groups {
    max-height: 300px;
    overflow: auto;
    margin-top: 12px;
    padding: 4px 6px 8px 2px;
    display: grid;
    gap: 16px;
  }

  h3 {
    font-size: 0.9rem;
    font-weight: 650;
    color: var(--muted);
    margin-bottom: 8px;
  }

  .items {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .item {
    font: inherit;
    font-weight: 560;
    color: var(--ink);
    background: var(--key);
    border: 1px solid var(--control-edge);
    border-radius: var(--r-key);
    min-width: 52px;
    min-height: 40px;
    padding: 4px 12px;
    cursor: pointer;
    box-shadow: 0 3px 0 var(--key-edge);
    transition:
      transform 120ms ease,
      box-shadow 120ms ease;
  }

  .item:hover:not(:disabled) {
    transform: translateY(-1px);
  }

  .item:active:not(:disabled) {
    transform: translateY(2px);
    box-shadow: 0 1px 0 var(--key-edge);
  }

  .item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .item.current {
    background: var(--accent);
    color: var(--on-accent);
    border-color: var(--ink);
    box-shadow: 0 3px 0 var(--accent-edge);
  }

  @media (max-width: 900px) {
    .assign {
      grid-template-columns: 1fr;
    }
  }
</style>
