<script lang="ts">
  import "katex/dist/katex.min.css";
  import CopyButton from "../components/CopyButton.svelte";
  import Panel from "../components/Panel.svelte";
  import PanelHeader from "../components/PanelHeader.svelte";
  import { MODEL_V1_RENDERED_HTML, MODEL_V1_SOURCE_MARKDOWN } from "../lib/generated/modelV1";
  import type { LangTag } from "../lib/types";

  function detectBrowserLang(): LangTag {
    if (typeof navigator === "undefined") {
      return "zh";
    }

    const preferred = Array.isArray(navigator.languages)
      ? [...navigator.languages, navigator.language]
      : [navigator.language];

    for (const tag of preferred) {
      const normalized = tag.trim().toLowerCase();
      if (normalized.startsWith("zh")) {
        return "zh";
      }
      if (normalized.startsWith("en")) {
        return "en";
      }
    }

    return "zh";
  }

  let lang = $state<LangTag>(detectBrowserLang());

  function t(zh: string, en: string): string {
    return lang === "zh" ? zh : en;
  }
</script>

<Panel>
  {#snippet header()}
    <PanelHeader
      titleText={t("这个 App 如何工作", "How this app works")}
      subtitleText={t(
        "不用看了，右边复制原文给 ChatGPT",
        "Don't look, just copy the text to ChatGPT, copy button on the right",
      )}
    >
      {#snippet controls()}
        <CopyButton {lang} text={MODEL_V1_SOURCE_MARKDOWN} />
      {/snippet}
    </PanelHeader>
  {/snippet}

  <section class="content" aria-label="model_v1.md">
    <article class="markdown-body">{@html MODEL_V1_RENDERED_HTML}</article>
  </section>
</Panel>

<style>
  .content {
    min-width: 0;
  }
</style>
