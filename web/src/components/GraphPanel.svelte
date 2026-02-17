<script lang="ts">
  import {
    Background,
    Controls,
    MiniMap,
    SvelteFlow,
    type Edge,
    type Node,
  } from "@xyflow/svelte";
  import { buildFlowGraph } from "../lib/graph";
  import type { LangTag, SolvePayload } from "../lib/types";

  interface Props {
    lang: LangTag;
    result: SolvePayload | null;
    graphFilter: "all" | string;
    onGraphFilterChange: (nextFilter: "all" | string) => void;
  }

  let { lang, result, graphFilter, onGraphFilterChange }: Props = $props();

  const flow = $derived<{ nodes: Node[]; edges: Edge[] }>(
    result
      ? buildFlowGraph(result.logisticsGraph, graphFilter)
      : { nodes: [], edges: [] },
  );

  function t(zh: string, en: string): string {
    return lang === "zh" ? zh : en;
  }
</script>

<div class="sub-header">
  <div>
    <h2>{t("物流图", "Logistics Graph")}</h2>
    <p class="subtitle">
      {t(
        "用于定位产销链路和流量集中区域。",
        "Inspect flow concentration and route direction.",
      )}
    </p>
  </div>

  {#if result}
    <select
      value={graphFilter}
      onchange={(event) => {
        onGraphFilterChange((event.currentTarget as HTMLSelectElement).value);
      }}
    >
      <option value="all">{t("全部物品", "All items")}</option>
      {#each result.logisticsGraph.items as item}
        <option value={item.itemKey}>{item.itemName}</option>
      {/each}
    </select>
  {/if}
</div>

{#if !result}
  <p class="hint">
    {t("求解后这里显示物流网络。", "Logistics network appears after solving.")}
  </p>
{:else}
  <div class="flow-wrap">
    <SvelteFlow nodes={flow.nodes} edges={flow.edges} fitView>
      <Background bgColor="#f9fcfa" patternColor="#d8e6de" gap={24} />
      <MiniMap pannable zoomable />
      <Controls />
    </SvelteFlow>
  </div>
{/if}

<style>
  .sub-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .subtitle {
    margin-top: 2px;
    color: var(--ink-soft);
    font-size: 12px;
  }

  .flow-wrap {
    border: 1px solid color-mix(in srgb, var(--line) 90%, #fff);
    border-radius: 12px;
    min-height: 560px;
    height: 72vh;
    overflow: hidden;
    background: #f9fcfa;
  }

  select {
    border: 1px solid #bfd3c9;
    border-radius: 8px;
    padding: 7px 9px;
    background: #fff;
    color: inherit;
  }

  .hint {
    margin: 0;
    color: var(--ink-soft);
  }

  @media (max-width: 760px) {
    .flow-wrap {
      min-height: 420px;
      height: 62vh;
    }
  }
</style>
