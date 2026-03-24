// This file is the explicit registry for Material Symbols icons.
// Add icons here to include them in the font subset.
// The type RegisteredIconName provides compile-time safety.

export const REGISTERED_ICONS = [
  "add",
  "analytics",
  "apps",
  "check",
  "check_circle",
  "close",
  "content_copy",
  "delete",
  "download",
  "draw",
  "error",
  "expand_more",
  "flowchart",
  "fullscreen",
  "fullscreen_exit",
  "help",
  "horizontal_rule",
  "info",
  "more_vert",
  "psychology",
  "share",
  "upload",
  "warning",
] as const;

export type RegisteredIconName = (typeof REGISTERED_ICONS)[number];
