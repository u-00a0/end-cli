import { expect, test } from '@playwright/test';

test('How it works page should not scroll at <html> level', async ({ page }) => {
  await page.goto('/#/how');

  await expect(page.getByRole('heading', { name: /How this app works|这个 App 如何工作/ })).toBeVisible();

  const debug = await page.evaluate(() => {
    const se = document.scrollingElement || document.documentElement;
    const shell = document.querySelector('.shell');

    const describe = (el) => {
      if (!el || !(el instanceof Element)) return null;
      const id = el.id ? `#${el.id}` : '';
      const cls = el.classList && el.classList.length > 0 ? `.${Array.from(el.classList).join('.')}` : '';
      return `${el.tagName}${id}${cls}`;
    };

    const shellRect = shell ? shell.getBoundingClientRect() : null;

    let maxEl = document.body;
    let maxBottom = -Infinity;

    for (const el of document.querySelectorAll('body *')) {
      const cs = getComputedStyle(el);
      if (cs.position === 'fixed') continue;

      const r = el.getBoundingClientRect();
      const bottom = r.bottom + window.scrollY;
      if (bottom > maxBottom) {
        maxBottom = bottom;
        maxEl = el;
      }
    }

    const maxElRect = maxEl.getBoundingClientRect();
    const maxElComputed = getComputedStyle(maxEl);

    return {
      url: location.href,
      viewport: { width: window.innerWidth, height: window.innerHeight },
      scrollingElement: describe(se),
      html: {
        clientHeight: se.clientHeight,
        scrollHeight: se.scrollHeight,
        overflowY: getComputedStyle(document.documentElement).overflowY,
        extraPx: se.scrollHeight - se.clientHeight,
      },
      body: {
        clientHeight: document.body.clientHeight,
        scrollHeight: document.body.scrollHeight,
        overflowY: getComputedStyle(document.body).overflowY,
      },
      shell: shellRect
        ? {
            el: describe(shell),
            top: shellRect.top,
            bottom: shellRect.bottom,
            height: shellRect.height,
          }
        : null,
      maxBottom: {
        el: describe(maxEl),
        bottom: maxBottom,
        rect: {
          top: maxElRect.top,
          bottom: maxElRect.bottom,
          height: maxElRect.height,
        },
        computed: {
          position: maxElComputed.position,
          display: maxElComputed.display,
          overflowY: maxElComputed.overflowY,
          height: maxElComputed.height,
          minHeight: maxElComputed.minHeight,
          marginTop: maxElComputed.marginTop,
          marginBottom: maxElComputed.marginBottom,
        },
      },
    };
  });

  console.log('HowItWorks scroll debug:', JSON.stringify(debug, null, 2));

  expect(debug.html.overflowY).toBe('hidden');
  expect(debug.body.overflowY).toBe('hidden');

  await page.mouse.wheel(0, 2000);
  const afterScroll = await page.evaluate(() => ({
    scrollY: window.scrollY,
    htmlScrollTop: document.documentElement.scrollTop,
    bodyScrollTop: document.body.scrollTop,
  }));
  expect(afterScroll.scrollY).toBe(0);
  expect(afterScroll.htmlScrollTop).toBe(0);
  expect(afterScroll.bodyScrollTop).toBe(0);
});
