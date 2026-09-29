<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/strings';
  import type { ProfileInfo } from '../lib/types';
  import Dialog from './Dialog.svelte';

  let saveOpen = $state(false);
  let saveName = $state('');
  let dupOpen = $state(false);
  let dupFrom = $state('');
  let dupName = $state('');
  let delOpen = $state(false);
  let delName = $state('');

  const connected = $derived(app.phase === 'ready' && !!app.info);
  const exists = (name: string) => app.profiles.some((p) => p.name.toLowerCase() === name.trim().toLowerCase());

  function applyState(p: ProfileInfo): { ok: boolean; reason: string } {
    if (!connected) return { ok: false, reason: t.profiles.needKeyboard };
    if (p.model !== app.info!.modelId) return { ok: false, reason: t.profiles.otherModel };
    return { ok: true, reason: '' };
  }

  async function submitSave(e: Event) {
    e.preventDefault();
    if (!saveName.trim()) return;
    saveOpen = false;
    await app.saveProfile(saveName);
  }

  async function submitDup(e: Event) {
    e.preventDefault();
    if (!dupName.trim()) return;
    dupOpen = false;
    await app.duplicateProfile(dupFrom, dupName);
  }

  async function confirmDelete() {
    delOpen = false;
    await app.deleteProfile(delName);
  }
</script>

<section class="profiles">
  <header class="head">
    <h1>{t.profiles.title}</h1>
    <div class="actions">
      <button class="btn" onclick={() => app.importProfile()}>{t.profiles.import}</button>
      <button
        class="btn primary"
        disabled={!connected}
        onclick={() => {
          saveName = '';
          saveOpen = true;
        }}>{t.profiles.saveAs}</button
      >
    </div>
  </header>

  {#if app.profileNote}
    <p class="note" class:error-text={app.profileNote.kind === 'error'} class:ok-text={app.profileNote.kind === 'ok'} role={app.profileNote.kind === 'error' ? 'alert' : 'status'}>
      {app.profileNote.text}
    </p>
  {/if}

  {#if app.profilesPhase === 'loading'}
    <p class="muted" role="status">{t.profiles.loading}</p>
  {:else if app.profilesPhase === 'error'}
    <div class="panel" role="alert">
      <p class="error-text">{app.profilesError}</p>
      <button class="btn" onclick={() => app.refreshProfiles()}>{t.state.retry}</button>
    </div>
  {:else if app.profiles.length === 0}
    <div class="empty panel">
      <h2>{t.profiles.empty.title}</h2>
      <p>{t.profiles.empty.body}</p>
      {#if !connected}<p class="hint">{t.profiles.needKeyboard}</p>{/if}
    </div>
  {:else}
    <ul class="list">
      {#each app.profiles as p (p.name)}
        {@const can = applyState(p)}
        <li>
          <div class="who">
            <div class="name">{p.name}</div>
            <div class="hint">{t.profiles.meta(p.model, p.layers)}{can.ok ? '' : `. ${can.reason}`}</div>
          </div>
          <div class="row-actions">
            <button class="btn small primary" disabled={!can.ok} onclick={() => app.applyProfile(p.name)}>{t.profiles.apply}</button>
            <button
              class="btn small"
              onclick={() => {
                dupFrom = p.name;
                dupName = `${p.name} copy`;
                dupOpen = true;
              }}>{t.profiles.duplicate}</button
            >
            <button class="btn small" onclick={() => app.exportProfile(p.name)}>{t.profiles.export}</button>
            <button
              class="btn small danger"
              onclick={() => {
                delName = p.name;
                delOpen = true;
              }}>{t.profiles.delete}</button
            >
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<Dialog bind:open={saveOpen} title={t.profiles.saveTitle}>
  <form onsubmit={submitSave} class="form">
    <label class="field">
      {t.profiles.nameLabel}
      <input class="input" bind:value={saveName} maxlength="60" autocomplete="off" />
    </label>
    {#if app.changes.keys > 0}<p class="hint">{t.profiles.unapplied(app.changes.keys)}</p>{/if}
    {#if saveName.trim() && exists(saveName)}<p class="hint">{t.profiles.overwrite(saveName.trim())}</p>{/if}
    <div class="buttons">
      <button type="button" class="btn" onclick={() => (saveOpen = false)}>{t.profiles.cancel}</button>
      <button type="submit" class="btn primary" disabled={!saveName.trim()}>{t.profiles.save}</button>
    </div>
  </form>
</Dialog>

<Dialog bind:open={dupOpen} title={t.profiles.duplicateTitle(dupFrom)}>
  <form onsubmit={submitDup} class="form">
    <label class="field">
      {t.profiles.nameLabel}
      <input class="input" bind:value={dupName} maxlength="60" autocomplete="off" />
    </label>
    {#if dupName.trim() && exists(dupName)}<p class="hint error-text">{t.profiles.overwrite(dupName.trim())}</p>{/if}
    <div class="buttons">
      <button type="button" class="btn" onclick={() => (dupOpen = false)}>{t.profiles.cancel}</button>
      <button type="submit" class="btn primary" disabled={!dupName.trim() || exists(dupName)}>{t.profiles.save}</button>
    </div>
  </form>
</Dialog>

<Dialog bind:open={delOpen} title={t.profiles.delete}>
  <p>{t.profiles.confirmDelete(delName)}</p>
  <div class="buttons">
    <button class="btn" onclick={() => (delOpen = false)}>{t.profiles.cancel}</button>
    <button class="btn danger" onclick={confirmDelete}>{t.profiles.confirm}</button>
  </div>
</Dialog>

<style>
  .profiles {
    display: grid;
    gap: 18px;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .actions,
  .row-actions,
  .buttons {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .buttons {
    justify-content: flex-end;
    margin-top: 16px;
  }

  .note {
    font-weight: 560;
  }

  .empty {
    display: grid;
    gap: 8px;
    max-width: 560px;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--line);
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding: 16px 4px;
    border-bottom: 1px solid var(--line);
  }

  .name {
    font-weight: 650;
    font-size: 1.05rem;
  }

  .form {
    display: grid;
    gap: 12px;
  }
</style>
