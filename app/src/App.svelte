<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from './lib/app.svelte';
  import { theme } from './lib/theme.svelte';
  import DeviceView from './components/DeviceView.svelte';
  import KeysView from './components/KeysView.svelte';
  import LightingView from './components/LightingView.svelte';
  import ProfilesView from './components/ProfilesView.svelte';
  import StateScreen from './components/StateScreen.svelte';
  import TopBar from './components/TopBar.svelte';

  onMount(() => {
    theme.init();
    void app.start();
  });

  const needsKeyboard = $derived(app.tab === 'keys' || app.tab === 'lighting');
</script>

<TopBar />

<main id="main" tabindex="-1">
  <div role="tabpanel" id="panel-{app.tab}" aria-labelledby="tab-{app.tab}">
    {#if needsKeyboard && app.phase !== 'ready'}
      <StateScreen
        kind={app.phase as 'searching' | 'loading' | 'error'}
        error={app.error}
        layers={app.info?.layers ?? 0}
        onretry={() => app.connect(false)}
        ondemo={() => app.connect(true)}
      />
    {:else if app.tab === 'keys'}
      <KeysView />
    {:else if app.tab === 'lighting'}
      <LightingView />
    {:else if app.tab === 'profiles'}
      <ProfilesView />
    {:else}
      <DeviceView />
    {/if}
  </div>
</main>

<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{app.announcement}</div>

<style>
  main {
    max-width: 1240px;
    margin: 0 auto;
    padding: 24px 28px 8px;
    outline: none;
  }

  @media (max-width: 900px) {
    main {
      padding: 16px;
    }
  }
</style>
