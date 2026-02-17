<script lang="ts">
  import type { LangTag, SolvePayload } from "../lib/types";

  interface Props {
    lang: LangTag;
    isBootstrapping: boolean;
    result: SolvePayload | null;
    statusMessage: string;
    errorMessage: string;
  }

  let { lang, isBootstrapping, result, statusMessage, errorMessage }: Props =
    $props();

  function t(zh: string, en: string): string {
    return lang === "zh" ? zh : en;
  }
</script>

<div class="panel-head">
  <div>
    <h2>{t("求解结果", "Solver Output")}</h2>
    <p class="subtitle">
      {t(
        "追踪收益、电力和产线规模变化。",
        "Track revenue, power balance, and line size.",
      )}
    </p>
  </div>

  <div class="solve-message">
    {#if statusMessage}
      <p class="status">{statusMessage}</p>
    {/if}
    {#if errorMessage}
      <p class="error">{errorMessage}</p>
    {/if}
  </div>
</div>

{#if isBootstrapping}
  <p class="hint">
    {t("正在加载 wasm 与基础数据...", "Loading wasm and baseline data...")}
  </p>
{:else if !result}
  <p class="hint">
    {t(
      "编辑参数后会自动求解。",
      "Results are solved automatically after edits.",
    )}
  </p>
{:else}
  <div class="kpi-grid">
    <article>
      <h3>{t("收益 / min", "Revenue / min")}</h3>
      <p>{result.summary.stage2RevenuePerMin.toFixed(2)}</p>
    </article>
    <article>
      <h3>{t("收益 / h", "Revenue / h")}</h3>
      <p>{result.summary.stage2RevenuePerHour.toFixed(0)}</p>
    </article>
    <article>
      <h3>{t("生产机器", "Machines")}</h3>
      <p>{result.summary.totalMachines}</p>
    </article>
    <article>
      <h3>{t("热容池", "Thermal Banks")}</h3>
      <p>{result.summary.totalThermalBanks}</p>
    </article>
    <article>
      <h3>{t("发电/用电", "Power Gen/Use")}</h3>
      <p>{result.summary.powerGenW}/{result.summary.powerUseW} W</p>
    </article>
    <article>
      <h3>{t("电力余量", "Power Margin")}</h3>
      <p>{result.summary.powerMarginW} W</p>
    </article>
  </div>

  <div class="table-wrap">
    <h3>{t("据点收益", "Outpost Revenue")}</h3>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>{t("据点", "Outpost")}</th>
            <th>{t("收益/min", "Value/min")}</th>
            <th>{t("上限/min", "Cap/min")}</th>
            <th>{t("占比", "Ratio")}</th>
          </tr>
        </thead>
        <tbody>
          {#each result.summary.outposts as outpost}
            <tr>
              <td>{outpost.name}</td>
              <td>{outpost.valuePerMin.toFixed(2)}</td>
              <td>{outpost.capPerMin.toFixed(2)}</td>
              <td>{(outpost.ratio * 100).toFixed(1)}%</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <div class="table-wrap">
    <h3>{t("高价值销售", "Top Sales")}</h3>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>{t("物品", "Item")}</th>
            <th>{t("据点", "Outpost")}</th>
            <th>{t("收益/min", "Value/min")}</th>
          </tr>
        </thead>
        <tbody>
          {#each result.summary.topSales.slice(0, 12) as sale}
            <tr>
              <td>{sale.itemName}</td>
              <td>{sale.outpostName}</td>
              <td>{sale.valuePerMin.toFixed(2)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  {#if result.summary.facilities.length > 0}
    <div class="table-wrap">
      <h3>{t("设施负载", "Facility Load")}</h3>
      <div class="table-scroll compact">
        <table>
          <thead>
            <tr>
              <th>{t("设施", "Facility")}</th>
              <th>{t("机器数", "Machines")}</th>
            </tr>
          </thead>
          <tbody>
            {#each result.summary.facilities.slice(0, 16) as facility}
              <tr>
                <td>{facility.name}</td>
                <td>{facility.machines}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
{/if}

<style>
  .panel-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    flex-wrap: wrap;
  }

  .subtitle {
    margin-top: 2px;
    color: var(--ink-soft);
    font-size: 12px;
  }

  .solve-message {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .status {
    margin: 0;
    color: #0b6f5a;
    font-size: 12px;
    font-weight: 500;
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
    font-weight: 600;
  }

  .kpi-grid {
    display: grid;
    gap: 8px;
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .kpi-grid article {
    border-radius: 10px;
    background: linear-gradient(170deg, #ffffff 0%, #f8fcf9 100%);
    border: 1px solid color-mix(in srgb, var(--line) 88%, white);
    padding: 10px;
  }

  .kpi-grid h3 {
    font-size: 12px;
    color: var(--ink-soft);
    margin-bottom: 8px;
  }

  .kpi-grid p {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .table-wrap {
    display: grid;
    gap: 8px;
    min-height: 0;
    min-width: 0;
  }

  .table-scroll {
    border: 1px solid color-mix(in srgb, var(--line) 86%, #fff);
    border-radius: 10px;
    overflow: auto;
    max-height: clamp(180px, 28dvh, 360px);
    background: #fff;
    min-width: 0;
    max-width: 100%;
  }

  .table-scroll.compact {
    max-height: clamp(160px, 22dvh, 280px);
  }

  table {
    width: max-content;
    min-width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th,
  td {
    border-bottom: 1px solid #dde8e1;
    text-align: left;
    padding: 8px 6px;
    overflow-wrap: anywhere;
  }

  th {
    color: var(--ink-soft);
    font-weight: 600;
  }

  .hint {
    margin: 0;
    color: var(--ink-soft);
  }

  @media (max-width: 1200px) {
    .kpi-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 760px) {
    .kpi-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
