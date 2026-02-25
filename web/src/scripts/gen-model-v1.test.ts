import { describe, expect, it } from 'vitest';

import { replaceEscapedDollarInsideMath } from '../../scripts/gen-model-v1.mjs';

describe('replaceEscapedDollarInsideMath', () => {
  it('keeps plain-text \\$ unchanged', () => {
    const input = String.raw`Price is \$100.`;
    expect(replaceEscapedDollarInsideMath(input)).toBe(input);
  });

  it('rewrites \\$ inside inline math to \\char36', () => {
    const input = String.raw`Inline: $s^{\$} \ge 0$`;
    const output = replaceEscapedDollarInsideMath(input);
    expect(output).toContain(String.raw`$s^{\char36} \ge 0$`);
  });

  it('rewrites \\$ inside block math to \\char36', () => {
    const input = String.raw`$$
\max\ s^{\$} \\
$$`;
    const output = replaceEscapedDollarInsideMath(input);
    expect(output).toContain(String.raw`s^{\char36}`);
    expect(output).not.toContain(String.raw`s^{\$}`);
  });

  it('does not touch fenced code blocks', () => {
    const input = String.raw`\`\`\`tex
$s^{\$}$
\`\`\``;
    expect(replaceEscapedDollarInsideMath(input)).toBe(input);
  });

  it('does not touch inline code spans', () => {
    const input = String.raw`Use \`$s^{\$}$\` as a snippet.`;
    expect(replaceEscapedDollarInsideMath(input)).toBe(input);
  });
});
