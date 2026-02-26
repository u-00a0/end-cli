<script lang="ts">
  import { onDestroy } from "svelte";
  import IconActionButton from "./IconActionButton.svelte";
  import { copyTextToClipboard } from "../lib/clipboard";
  import type { LangTag } from "../lib/types";

  interface Props {
    lang: LangTag;
    text: string;
  }

  type CopyState = "idle" | "copied" | "failed";

  let { lang, text }: Props = $props();

  let copyState = $state<CopyState>("idle");
  let copyStateTimerId: number | null = null;

  const isDisabled = $derived(text.trim().length === 0);

  const buttonLabel = $derived.by(() => {
    if (copyState === "copied") {
      return t("已复制", "Copied");
    }

    if (copyState === "failed") {
      return t("复制失败", "Copy failed");
    }

    return isDisabled
      ? t("暂无可复制内容", "Nothing to copy")
      : t("复制输出", "Copy output");
  });

  function t(zh: string, en: string): string {
    return lang === "zh" ? zh : en;
  }

  function resetCopyStateLater(): void {
    if (copyStateTimerId !== null && typeof window !== "undefined") {
      window.clearTimeout(copyStateTimerId);
    }

    if (typeof window === "undefined") {
      return;
    }

    copyStateTimerId = window.setTimeout(() => {
      copyState = "idle";
      copyStateTimerId = null;
    }, 1400);
  }

  async function copyOutput(): Promise<void> {
    if (isDisabled) {
      return;
    }

    try {
      await copyTextToClipboard(text);
      copyState = "copied";
    } catch {
      copyState = "failed";
    }
    resetCopyStateLater();
  }

  onDestroy(() => {
    if (copyStateTimerId !== null && typeof window !== "undefined") {
      window.clearTimeout(copyStateTimerId);
      copyStateTimerId = null;
    }
  });
</script>

<IconActionButton
  icon={copyState === "copied" ? "check" : "content_copy"}
  onClick={() => {
    void copyOutput();
  }}
  disabled={isDisabled}
  ariaLabel={buttonLabel}
  title={buttonLabel}
/>
