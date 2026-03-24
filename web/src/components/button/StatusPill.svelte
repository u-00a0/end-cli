<script lang="ts">
  import { translateByLang } from "../../lib/lang";
  import type { LangTag } from "../../lib/types";
  import { tooltip } from "../../lib/tooltip";
  import MaterialSymbol from "../icon/MaterialSymbol.svelte";

  export type SolveStatusPillState =
    | { status: "solving"; elapsedMs: number | null }
    | { status: "ok"; elapsedMs: number | null }
    | { status: "error"; elapsedMs: number | null };

  interface Props {
    state: SolveStatusPillState;
    lang: LangTag;
  }

  let { state, lang }: Props = $props();

  const isDanger = $derived(state.status === "error");

  function t(zh: string, en: string): string {
    return translateByLang(lang, zh, en);
  }

  function formatElapsed(ms: number | null): string {
    if (ms === null) {
      // unicode em dash, looks better than `-`
      return "— ms";
    }

    if (ms < 1000) {
      return `${ms} ms`;
    }

    const seconds = ms / 1000;
    return `${seconds.toFixed(seconds < 10 ? 2 : 1)} s`;
  }

  const tooltipText = $derived.by((): string => {
    const elapsedText = formatElapsed(state.elapsedMs);

    if (state.status === "solving") {
      return t("求解进行中，耗时 ", "Solving, elapsed ") + elapsedText;
    }

    if (state.status === "error") {
      return t("求解失败，请查看下方原因", "Failed, see details below");
    }

    return t("求解已成功，耗时 ", "Succeeded, elapsed ") + elapsedText;
  });

</script>

<div class="solve-meta" class:danger={isDanger} use:tooltip={tooltipText}>
  {#if state.status === "solving"}
    <span class="spinner" aria-hidden="true">
      <span class="spinner-inner"></span>
    </span>
  {:else if state.status === "error"}
    <span class="solve-icon danger" aria-hidden="true">
      <MaterialSymbol icon="error" size={16} weight={600} opsz={16} />
    </span>
  {:else}
    <span class="solve-icon" aria-hidden="true">
      <MaterialSymbol icon="check_circle" size={16} weight={600} opsz={16} />
    </span>
  {/if}

  <p class="elapsed" class:danger={isDanger}>{formatElapsed(state.elapsedMs)}</p>
</div>

<style>
  .solve-meta {
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    background: color-mix(in srgb,var(--surface-soft) 60%,var(--accent-soft));
    padding: 8px 12px;
    min-height: var(--control-size);
    contain: layout paint;
  }

  .solve-meta.danger {
    background: var(--danger-soft);
  }

  @media (hover: hover) and (pointer: fine) {
    .solve-meta:hover {
      background: var(--accent-soft);
    }

    .solve-meta.danger:hover {
      background: var(--danger-soft-hover);
    }
  }

  .spinner {
    width: 16px;
    height: 16px;
    display: grid;
    place-items: center;
    flex: 0 0 16px;
  }

  /* 为了匹配 check_circle 的外圈，这里需要 14x14 px */
  .spinner-inner {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid color-mix(in srgb, var(--line) 80%, var(--line-tint-3));
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
    flex: 0 0 auto;
  }

  .solve-icon {
    color: var(--accent);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    flex: 0 0 16px;
  }

  .solve-icon.danger {
    color: var(--danger);
  }

  .elapsed {
    margin: 0;
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
    width: 52px;
    text-align: right;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .elapsed.danger {
    color: var(--danger);
  }

  @keyframes spin {
    to {
      transform: rotate(1turn);
    }
  }
</style>
