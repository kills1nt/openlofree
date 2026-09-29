<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/strings';

  let reading = $state(false);
  const info = $derived(app.info);
  const hex4 = (n: number) => n.toString(16).toUpperCase().padStart(4, '0');

  async function again() {
    reading = true;
    await app.load();
    reading = false;
  }
</script>

<section class="device">
  <h1>{t.device.title}</h1>

  {#if !info || app.phase !== 'ready'}
    <div class="panel offline">
      <p>{t.device.offline}</p>
      <div class="actions">
        <button class="btn primary" onclick={() => app.connect(false)}>{t.state.retry}</button>
        <button class="btn" onclick={() => app.connect(true)}>{t.state.demo}</button>
      </div>
      <p class="hint">{t.state.demoNote}</p>
    </div>
  {:else}
    <div class="grid">
      <div class="panel">
        <dl>
          <dt>{t.device.model}</dt>
          <dd>
            {info.label}
            {#if !info.verified}<span class="hint"> {t.device.unverified}</span>{/if}
          </dd>
          <dt>{t.device.connection}</dt>
          <dd>{info.demo ? t.device.demo : t.device.usb}</dd>
          <dt>{t.device.product}</dt>
          <dd>{info.product}</dd>
          <dt>{t.device.ids}</dt>
          <dd>{hex4(info.vendorId)}:{hex4(info.productId)}</dd>
          <dt>{t.device.protocol}</dt>
          <dd>{info.protocol}</dd>
          <dt>{t.device.layers}</dt>
          <dd>{info.layers}</dd>
          <dt>{t.device.backup}</dt>
          <dd>
            {#if info.demo}
              {t.device.backupDemo}
            {:else if app.backup?.exists && app.backup.path}
              <span class="path">{app.backup.path}</span>
            {:else}
              {t.device.backupMissing}
            {/if}
          </dd>
        </dl>
        <div class="actions">
          <button class="btn" onclick={again} disabled={reading}>{reading ? '…' : t.device.refresh}</button>
          <button class="btn" onclick={() => app.disconnect(info.demo)}>{info.demo ? t.device.leaveDemo : t.device.disconnect}</button>
        </div>
      </div>

      <div class="panel battery">
        <h2>{t.device.battery}</h2>
        {#if app.battery !== null}
          <div class="big" aria-label="{app.battery} percent">{app.battery}<span>%</span></div>
        {:else}
          <div class="big none">{t.device.batteryUnknown}</div>
          <p class="hint">{t.device.batteryWhy}</p>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .device {
    display: grid;
    gap: 18px;
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 20px;
    align-items: start;
  }

  dl {
    display: grid;
    grid-template-columns: 150px 1fr;
    gap: 12px 16px;
    margin: 0 0 20px;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-weight: 560;
    overflow-wrap: anywhere;
  }

  .path {
    font-weight: 400;
    font-size: 0.9rem;
  }

  .actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .offline {
    display: grid;
    gap: 12px;
    max-width: 520px;
  }

  .battery {
    display: grid;
    gap: 10px;
  }

  .big {
    font-size: 3.4rem;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.02em;
  }

  .big span {
    font-size: 1.6rem;
    margin-left: 2px;
  }

  .big.none {
    font-size: 1.3rem;
    letter-spacing: 0;
  }

  @media (max-width: 900px) {
    .grid {
      grid-template-columns: 1fr;
    }

    dl {
      grid-template-columns: 1fr;
      gap: 2px;
    }

    dd {
      margin-bottom: 10px;
    }
  }
</style>
