<script lang="ts">
  import "katex/dist/katex.min.css";
  import CopyButton from "../components/CopyButton.svelte";
  import Panel from "../components/Panel.svelte";
  import PanelHeader from "../components/PanelHeader.svelte";
  import {
    MODEL_V1_RENDERED_HTML,
    MODEL_V1_SOURCE_MARKDOWN,
  } from "../lib/generated/modelV1";
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
    display: flex;
    justify-content: center;
  }

  /* The Markdown HTML is injected via {@html ...}, so styles must be global. */
  .content :global(.markdown-body) {
    width: 100%;
    max-width: 960px;
    margin: 0 auto;
    padding: clamp(10px, 1.4vw, 18px) clamp(12px, 2vw, 22px);
    border-radius: var(--radius-lg);
    border: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
    background: color-mix(in srgb, var(--surface-soft) 82%, transparent);

    color: var(--ink);
    font-size: 14px;
    line-height: 1.68;
    word-break: break-word;
  }

  @media (min-width: 760px) {
    .content :global(.markdown-body) {
      font-size: 16px;
    }
  }

  .content :global(.katex) {
    /* open PR as 2026/02/26: https://github.com/KaTeX/KaTeX/pull/3859 */
    /* katex hidden mathml uses absolute, cause issue when parent scrolling element is non-static */
    /* here we do a minimal fix by setting .katex position to relative */
    /* 2 hr and lots of tokens spent debugging this --xks */
    position: relative;
  }

  .content :global(.markdown-body > :first-child) {
    margin-top: 0;
  }

  .content :global(.markdown-body > :last-child) {
    margin-bottom: 0;
  }

  .content :global(.markdown-body p) {
    margin: var(--space-3) 0;
  }

  .content :global(.markdown-body h1),
  .content :global(.markdown-body h2),
  .content :global(.markdown-body h3),
  .content :global(.markdown-body h4) {
    margin: var(--space-5) 0 var(--space-3);
    line-height: 1.25;
    letter-spacing: -0.01em;
  }

  .content :global(.markdown-body h2) {
    padding-bottom: var(--space-2);
    border-bottom: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
  }

  .content :global(.markdown-body h3) {
    margin-top: var(--space-4);
  }

  .content :global(.markdown-body ul),
  .content :global(.markdown-body ol) {
    margin: var(--space-3) 0;
    padding-left: 1.25em;
  }

  .content :global(.markdown-body li) {
    margin: var(--space-2) 0;
  }

  .content :global(.markdown-body li > p) {
    margin: var(--space-2) 0;
  }

  .content :global(.markdown-body a) {
    color: var(--accent-ink);
    text-underline-offset: 0.18em;
  }

  .content :global(.markdown-body a:hover) {
    text-decoration-thickness: 2px;
  }

  .content :global(.markdown-body code) {
    font-size: 0.92em;
    padding: 0.14em 0.38em;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--surface-soft) 70%, transparent);
    border: 1px solid color-mix(in srgb, var(--line) 62%, transparent);
  }

  .content :global(.markdown-body pre) {
    margin: var(--space-4) 0;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface-soft);
    border: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
    overflow-x: auto;
  }

  .content :global(.markdown-body pre code) {
    padding: 0;
    border: none;
    background: transparent;
    font-size: 0.9em;
    line-height: 1.55;
  }

  .content :global(.markdown-body blockquote) {
    margin: var(--space-4) 0;
    padding: var(--space-3) var(--space-4);
    border-left: 4px solid color-mix(in srgb, var(--accent) 42%, var(--line));
    background: color-mix(in srgb, var(--accent-soft) 55%, transparent);
    border-radius: var(--radius-md);
  }

  .content :global(.markdown-body blockquote > :first-child) {
    margin-top: 0;
  }

  .content :global(.markdown-body blockquote > :last-child) {
    margin-bottom: 0;
  }

  .content :global(.markdown-body hr) {
    border: none;
    border-top: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
    margin: var(--space-5) 0;
  }

  .content :global(.markdown-body table) {
    width: 100%;
    border-collapse: collapse;
    margin: var(--space-4) 0;
    overflow: hidden;
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
  }

  .content :global(.markdown-body th),
  .content :global(.markdown-body td) {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid color-mix(in srgb, var(--line) 70%, transparent);
    vertical-align: top;
  }

  .content :global(.markdown-body th) {
    text-align: left;
    background: color-mix(in srgb, var(--surface-soft) 65%, transparent);
    font-weight: 600;
  }

  .content :global(.markdown-body tr:last-child td) {
    border-bottom: none;
  }

  /* KaTeX: keep display math readable on narrow screens. */
  .content :global(.katex-display) {
    margin: var(--space-4) 0;
    overflow-x: auto;
    overflow-y: hidden;
    padding: var(--space-2) 0;
  }

  .content :global(.katex-display > .katex) {
    max-width: 100%;
  }

  .content :global(.katex) {
    font-size: 1.02em;
  }

  .content :global(.markdown-body li + li) {
    margin-top: var(--space-2);
  }
</style>
