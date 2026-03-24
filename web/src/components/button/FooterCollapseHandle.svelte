<script lang="ts">
  import { tooltip } from "../../lib/tooltip";
  import MaterialSymbol from "../icon/MaterialSymbol.svelte";

  interface Props {
    isCollapsed: boolean;
    label: string;
    onToggle: () => void;
  }

  let { isCollapsed, label, onToggle }: Props = $props();
</script>

<button
  type="button"
  class="brand-handle"
  aria-label={label}
  use:tooltip={label}
  onclick={onToggle}
>
  <span class="handle-icon" class:rotated={isCollapsed} aria-hidden="true">
    <MaterialSymbol icon="expand_more" size={20} weight={400} opsz={20} />
  </span>
  <img src="/favicon.svg" alt="" width="24" height="24" aria-hidden="true" />
</button>

<style>
  .brand-handle {
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    color: inherit;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    height: var(--control-size);
    line-height: 1;
  }

  .brand-handle:hover {
    background: color-mix(in srgb, var(--surface-soft) 60%, var(--accent-soft));
  }

  .handle-icon {
    display: inline-flex;
    align-items: center;
    transition: transform 160ms ease;
    transform: rotate(0deg);
  }

  .handle-icon.rotated {
    transform: rotate(180deg);
  }
</style>
