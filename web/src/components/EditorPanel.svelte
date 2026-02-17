<script lang="ts">
  import type { EditorPanelProps } from "../lib/editor-actions";
  import type { OutpostDraft } from "../lib/types";

  let {
    lang,
    draft,
    catalogItems,
    selectedOutpostIndex,
    isResetDisabled,
    actions,
  }: EditorPanelProps = $props();

  const selectedOutpost = $derived<OutpostDraft | null>(
    draft.outposts[selectedOutpostIndex] ?? null,
  );

  function t(zh: string, en: string): string {
    return lang === "zh" ? zh : en;
  }

  function itemTitle(itemKey: string): string {
    const item = catalogItems.find((entry) => entry.key === itemKey);
    if (!item) {
      return itemKey;
    }
    return lang === "zh"
      ? `${item.zh} (${item.key})`
      : `${item.en} (${item.key})`;
  }
</script>

<section class="editor-shell">
  <div class="panel-head">
    <div>
      <h2>{t("配置编辑器", "Configuration Editor")}</h2>
      <p class="subtitle">
        {t(
          "输入参数决定自动求解目标空间。",
          "Input values define the optimization space.",
        )}
      </p>
    </div>
    <div class="panel-actions">
      <button
        class="danger icon-btn"
        onclick={actions.resetToDefault}
        disabled={isResetDisabled}
        aria-label={t("重置默认配置", "Reset Default")}
        title={t("重置默认配置", "Reset Default")}
      >
        <span class="material-symbols-outlined icon" aria-hidden="true"
          >delete_forever</span
        >
      </button>

      <label
        class="file-btn secondary icon-btn"
        aria-label={t("导入 aic.toml", "Import aic.toml")}
      >
        <span class="material-symbols-outlined icon" aria-hidden="true"
          >upload</span
        >
        <input
          type="file"
          accept=".toml,text/plain"
          onchange={actions.importFromFile}
        />
      </label>

      <button
        class="secondary icon-btn"
        onclick={actions.exportToml}
        aria-label={t("导出 aic.toml", "Export aic.toml")}
        title={t("导出 aic.toml", "Export aic.toml")}
      >
        <span class="material-symbols-outlined icon" aria-hidden="true"
          >download</span
        >
      </button>
    </div>
  </div>

  <div class="field-row">
    <label for="external-power">{t("额外耗电 (W)", "External Power (W)")}</label
    >
    <input
      id="external-power"
      type="number"
      min="0"
      value={draft.externalPowerConsumptionW}
      oninput={(event) => {
        actions.setExternalPower(
          Number((event.currentTarget as HTMLInputElement).value),
        );
      }}
    />
  </div>

  <article class="sub-panel">
    <div class="sub-header">
      <h3>{t("外部供给 / min", "External Supply / min")}</h3>
      <button class="tiny" onclick={actions.supply.add}>+</button>
    </div>

    {#if draft.supply.length === 0}
      <p class="hint">{t("暂无供给条目。", "No supply rows yet.")}</p>
    {/if}

    {#each draft.supply as row, rowIndex}
      <div class="row-grid">
        <select
          value={row.itemKey}
          onchange={(event) =>
            actions.supply.setKey(
              rowIndex,
              (event.currentTarget as HTMLSelectElement).value,
            )}
        >
          {#each catalogItems as item}
            <option value={item.key}>{itemTitle(item.key)}</option>
          {/each}
        </select>

        <input
          type="number"
          min="0"
          value={row.value}
          oninput={(event) =>
            actions.supply.setValue(
              rowIndex,
              Number((event.currentTarget as HTMLInputElement).value),
            )}
        />

        <button class="danger" onclick={() => actions.supply.remove(rowIndex)}
          >×</button
        >
      </div>
    {/each}
  </article>

  <article class="sub-panel">
    <div class="sub-header">
      <h3>{t("据点", "Outposts")}</h3>
      <button class="tiny" onclick={actions.outposts.add}>+</button>
    </div>

    {#if draft.outposts.length === 0}
      <p class="hint">{t("暂无据点。", "No outposts yet.")}</p>
    {/if}

    {#if draft.outposts.length > 0}
      <div class="outpost-layout">
        <div class="outpost-list">
          {#each draft.outposts as outpost, outpostIndex}
            <button
              type="button"
              class={`outpost-pick ${outpostIndex === selectedOutpostIndex ? "active" : ""}`}
              onclick={() => actions.outposts.select(outpostIndex)}
            >
              <p class="outpost-pick-title">
                {outpost.key || `${t("据点", "Outpost")} ${outpostIndex + 1}`}
              </p>
              <p class="outpost-pick-meta">
                {t("每小时交易上限", "Money Cap / h")}: {outpost.moneyCapPerHour}
              </p>
              <p class="outpost-pick-meta">
                {t("收购条目", "Price rows")}: {outpost.prices.length}
              </p>
            </button>
          {/each}
        </div>

        {#if selectedOutpost}
          <article class="outpost-card">
            <div class="outpost-head">
              <h4>
                {selectedOutpost.key ||
                  `${t("据点", "Outpost")} ${(selectedOutpostIndex >= 0 ? selectedOutpostIndex : 0) + 1}`}
              </h4>
              <button
                class="danger"
                onclick={() => actions.outposts.remove(selectedOutpostIndex)}
                >×</button
              >
            </div>

            <div class="row-grid two">
              <label>
                {t("Key", "Key")}
                <input
                  type="text"
                  value={selectedOutpost.key}
                  oninput={(event) =>
                    actions.outposts.setField(
                      selectedOutpostIndex,
                      "key",
                      (event.currentTarget as HTMLInputElement).value,
                    )}
                />
              </label>

              <label>
                EN
                <input
                  type="text"
                  value={selectedOutpost.en}
                  oninput={(event) =>
                    actions.outposts.setField(
                      selectedOutpostIndex,
                      "en",
                      (event.currentTarget as HTMLInputElement).value,
                    )}
                />
              </label>

              <label>
                ZH
                <input
                  type="text"
                  value={selectedOutpost.zh}
                  oninput={(event) =>
                    actions.outposts.setField(
                      selectedOutpostIndex,
                      "zh",
                      (event.currentTarget as HTMLInputElement).value,
                    )}
                />
              </label>

              <label>
                {t("每小时交易上限", "Money Cap / h")}
                <input
                  type="number"
                  min="0"
                  value={selectedOutpost.moneyCapPerHour}
                  oninput={(event) =>
                    actions.outposts.setField(
                      selectedOutpostIndex,
                      "moneyCapPerHour",
                      Number((event.currentTarget as HTMLInputElement).value),
                    )}
                />
              </label>
            </div>

            <div class="sub-header mini">
              <h5>{t("收购价", "Price Table")}</h5>
              <button
                class="tiny"
                onclick={() => actions.prices.add(selectedOutpostIndex)}
                >+</button
              >
            </div>

            {#if selectedOutpost.prices.length === 0}
              <p class="hint">{t("暂无价格条目。", "No price rows yet.")}</p>
            {/if}

            {#each selectedOutpost.prices as price, priceIndex}
              <div class="row-grid">
                <select
                  value={price.itemKey}
                  onchange={(event) =>
                    actions.prices.setKey(
                      selectedOutpostIndex,
                      priceIndex,
                      (event.currentTarget as HTMLSelectElement).value,
                    )}
                >
                  {#each catalogItems as item}
                    <option value={item.key}>{itemTitle(item.key)}</option>
                  {/each}
                </select>

                <input
                  type="number"
                  min="0"
                  value={price.price}
                  oninput={(event) =>
                    actions.prices.setValue(
                      selectedOutpostIndex,
                      priceIndex,
                      Number((event.currentTarget as HTMLInputElement).value),
                    )}
                />

                <button
                  class="danger"
                  onclick={() =>
                    actions.prices.remove(selectedOutpostIndex, priceIndex)}
                  >×</button
                >
              </div>
            {/each}
          </article>
        {/if}
      </div>
    {/if}
  </article>
</section>

<style>
  .editor-shell {
    display: grid;
    gap: 12px;
  }

  .panel-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    flex-wrap: wrap;
  }

  .subtitle {
    color: var(--ink-soft);
    font-size: 12px;
    margin-top: 3px;
  }

  .panel-actions {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .sub-panel {
    border: 1px solid color-mix(in srgb, var(--line) 82%, #fff);
    border-radius: 12px;
    padding: 11px;
    display: grid;
    gap: 9px;
    background: color-mix(in srgb, var(--panel) 84%, #fafffd);
  }

  .sub-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
  }

  .sub-header.mini {
    margin-top: 4px;
  }

  .field-row {
    display: grid;
    grid-template-columns: 1fr 190px;
    gap: 8px;
    align-items: center;
  }

  .row-grid {
    display: grid;
    gap: 8px;
    grid-template-columns: 1fr 140px 36px;
  }

  .row-grid.two {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .outpost-layout {
    display: grid;
    gap: 10px;
    grid-template-columns: minmax(220px, 280px) minmax(0, 1fr);
  }

  .outpost-list {
    display: grid;
    gap: 8px;
    align-content: start;
  }

  .outpost-pick {
    border: 1px solid #c9ddd1;
    border-radius: 10px;
    background: linear-gradient(170deg, #f7fcf8 0%, #f3faf6 100%);
    display: grid;
    gap: 3px;
    text-align: left;
    padding: 10px;
  }

  .outpost-pick.active {
    border-color: #52ab8f;
    background: linear-gradient(170deg, #ebf9f2 0%, #e0f5ec 100%);
    box-shadow: inset 0 0 0 1px #8ccdb8;
  }

  .outpost-pick-title {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .outpost-pick-meta {
    margin: 0;
    color: var(--ink-soft);
    font-size: 12px;
  }

  .outpost-card {
    border: 1px solid #cfe2d8;
    border-radius: 10px;
    padding: 10px;
    display: grid;
    gap: 8px;
    background: #fcfefc;
  }

  .outpost-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  input,
  select {
    border: 1px solid #bdd3c8;
    border-radius: 8px;
    padding: 7px 9px;
    background: #fff;
    color: inherit;
  }

  button,
  .file-btn {
    border: 1px solid #bdd3c8;
    border-radius: 8px;
    padding: 7px 9px;
    background: #fff;
    color: inherit;
    font: inherit;
    line-height: 1.2;
    cursor: pointer;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .secondary {
    background: #f8fbf9;
  }

  .file-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 36px;
  }

  .icon-btn {
    width: 36px;
    height: 36px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .icon {
    font-size: 20px;
    line-height: 1;
    display: block;
  }

  .file-btn input {
    display: none;
  }

  .tiny {
    width: 32px;
    padding: 0;
    font-size: 22px;
    line-height: 1;
    font-weight: 500;
  }

  .danger {
    color: var(--danger);
    border-color: #dca3b4;
    background: #fff5f8;
  }

  .hint {
    margin: 0;
    color: var(--ink-soft);
    font-size: 13px;
  }

  @media (max-width: 1200px) {
    .outpost-layout {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 760px) {
    .field-row {
      grid-template-columns: 1fr;
    }

    .row-grid {
      grid-template-columns: 1fr 104px 34px;
    }

    .row-grid.two {
      grid-template-columns: 1fr;
    }
  }
</style>
