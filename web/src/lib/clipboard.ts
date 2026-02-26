function fallbackCopyText(value: string): boolean {
  if (typeof document === "undefined") {
    return false;
  }

  try {
    const textarea = document.createElement("textarea");
    textarea.value = value;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    textarea.style.left = "-9999px";
    textarea.style.top = "0";
    document.body.appendChild(textarea);
    textarea.select();
    textarea.setSelectionRange(0, textarea.value.length);
    const ok = document.execCommand("copy");
    document.body.removeChild(textarea);
    return ok;
  } catch {
    return false;
  }
}

export async function copyTextToClipboard(text: string): Promise<void> {
  const value = text;
  if (value.trim().length === 0) {
    throw new Error("Nothing to copy");
  }

  if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(value);
      return;
    } catch {
      // fall back
    }
  }

  const ok = fallbackCopyText(value);
  if (!ok) {
    throw new Error("Copy failed");
  }
}

export async function copyPngBlobToClipboard(blob: Blob): Promise<void> {
  if (typeof navigator === "undefined") {
    throw new Error("Clipboard unavailable");
  }

  const ClipboardItemCtor = (globalThis as unknown as { ClipboardItem?: typeof ClipboardItem })
    .ClipboardItem;
  if (!navigator.clipboard?.write || !ClipboardItemCtor) {
    throw new Error("Image clipboard unsupported");
  }

  if (blob.type !== "image/png") {
    throw new Error("Expected PNG image");
  }

  await navigator.clipboard.write([
    new ClipboardItemCtor({ "image/png": blob }),
  ]);
}
