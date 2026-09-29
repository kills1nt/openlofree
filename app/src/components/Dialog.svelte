<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    title: string;
    children: Snippet;
  }

  let { open = $bindable(false), title, children }: Props = $props();
  let el: HTMLDialogElement | undefined = $state();
  const id = `dlg-${Math.random().toString(36).slice(2, 8)}`;

  $effect(() => {
    if (!el) return;
    if (open && !el.open) el.showModal();
    if (!open && el.open) el.close();
  });
</script>

<dialog bind:this={el} onclose={() => (open = false)} aria-labelledby={id}>
  <h2 {id}>{title}</h2>
  {@render children()}
</dialog>

<style>
  dialog {
    width: min(440px, calc(100vw - 32px));
    padding: 22px;
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--control-edge);
    border-radius: var(--r-panel);
  }

  dialog::backdrop {
    background: var(--scrim);
  }

  h2 {
    margin-bottom: 14px;
  }
</style>
