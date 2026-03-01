import type { Edge, Node } from "@xyflow/svelte";

export type FlowSnapshot = {
  nodes: Node[];
  edges: Edge[];
};
