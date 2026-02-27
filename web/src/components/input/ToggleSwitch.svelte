<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    ariaLabel: string;
    id?: string;
    disabled?: boolean;
    onToggle?: (nextValue: boolean) => void;
  }

  let {
    checked,
    label,
    ariaLabel,
    id,
    disabled = false,
    onToggle,
  }: Props = $props();

  function toggle(): void {
    if (disabled) {
      return;
    }

    onToggle?.(!checked);
  }
</script>

<button
  type="button"
  {id}
  class={`toggle-switch ${checked ? "is-checked" : ""}`}
  role="switch"
  aria-checked={checked}
  aria-label={ariaLabel}
  {disabled}
  onclick={toggle}
>
  <span class="track" aria-hidden="true">
    <span class="thumb"></span>
  </span>
  <span class="text">{label}</span>
</button>

<style>
  .toggle-switch {
    width: fit-content;
    max-width: 100%;
    border-radius: 999px;
    border: none;
    min-height: var(--control-size);
    padding: 0 10px 0 8px;
    background: var(--panel-strong);
    color: inherit;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }

  .toggle-switch:disabled {
    cursor: not-allowed;
    opacity: 0.65;
  }

  .track {
    width: 34px;
    height: 20px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--line) 78%, var(--line-tint-2));
    border: 1px solid color-mix(in srgb, var(--line) 90%, var(--line-tint-4));
    position: relative;
    flex: 0 0 auto;
    transition: background-color 140ms ease;
  }

  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: var(--panel-strong);
    box-shadow: 0 1px 2px color-mix(in srgb, var(--overlay-ink) 25%, transparent);
    transition: transform 200ms ease;
  }

  .is-checked .track {
    background: color-mix(in srgb, var(--accent) 76%, var(--accent-tint-2));
    border-color: color-mix(in srgb, var(--accent) 58%, var(--accent-tint-1));
  }

  .is-checked .thumb {
    transform: translateX(14px);
  }

  .text {
    font-size: 13px;
    line-height: 1.25;
    white-space: normal;
    text-align: left;
  }

  @media (hover: hover) and (pointer: fine) {
    .toggle-switch:hover:not(:disabled) {
      box-shadow: var(--focus-ring);
    }
  }

  .toggle-switch:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }
</style>
