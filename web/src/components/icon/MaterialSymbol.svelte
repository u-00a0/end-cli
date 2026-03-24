<script lang="ts">
  import { MATERIAL_SYMBOLS_CODEPOINTS } from "../../lib/generated/material-symbols-codepoints";
  import type { RegisteredIconName } from "../../lib/icon-registry";

  interface Props {
    icon: RegisteredIconName;
    class?: string;
    ariaHidden?: boolean;
    size?: number;
    fill?: 0 | 1;
    weight?: number;
    grad?: number;
    opsz?: number;
  }

  let {
    icon,
    class: className = "",
    ariaHidden = true,
    size = 24,
    fill = 0,
    weight = 400,
    grad = 0,
    opsz = size,
  }: Props = $props();

  const glyph = $derived(String.fromCodePoint(MATERIAL_SYMBOLS_CODEPOINTS[icon]));

  const iconClass = $derived.by(() =>
    ["material-symbols-outlined", className.trim()].filter(Boolean).join(" "),
  );

  const styleValue = $derived.by(
    () =>
      `font-size:${size}px;line-height:1;display:block;font-variation-settings:"FILL" ${fill},"wght" ${weight},"GRAD" ${grad},"opsz" ${opsz};`,
  );
</script>

<span
  class={iconClass}
  style={styleValue}
  aria-hidden={ariaHidden ? "true" : undefined}
>{glyph}</span>
