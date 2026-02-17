import type { Edge, Node } from '@xyflow/svelte';
import type { LogisticsGraphDto } from './types';

type FilterKey = string | 'all';

const kindColor: Record<string, string> = {
  external_supply: '#2f8f83',
  recipe_output: '#2a678a',
  recipe_input: '#6f5f9f',
  outpost_sale: '#b16d00',
  thermal_bank_fuel: '#8b305e'
};

function isSupplyKind(kind: string): boolean {
  return kind === 'external_supply' || kind === 'recipe_output';
}

function pickNodeColor(kind: string): string {
  return kindColor[kind] ?? '#3f5165';
}

export function buildFlowGraph(
  graph: LogisticsGraphDto,
  filterKey: FilterKey
): { nodes: Node[]; edges: Edge[] } {
  const edges =
    filterKey === 'all'
      ? graph.edges
      : graph.edges.filter((edge) => edge.itemKey === filterKey);

  const nodeIdSet = new Set<string>();
  for (const edge of edges) {
    nodeIdSet.add(edge.source);
    nodeIdSet.add(edge.target);
  }

  const nodes = graph.nodes.filter((node) => nodeIdSet.has(node.id));
  const supplyNodes = nodes.filter((node) => isSupplyKind(node.kind));
  const demandNodes = nodes.filter((node) => !isSupplyKind(node.kind));

  const positionedNodes: Node[] = [];
  const xSupply = 90;
  const xDemand = 510;
  const yStep = 86;

  supplyNodes.forEach((node, index) => {
    const color = pickNodeColor(node.kind);
    positionedNodes.push({
      id: node.id,
      data: {
        label: node.label
      },
      position: {
        x: xSupply,
        y: 24 + index * yStep
      },
      style:
        `border:1px solid ${color};` +
        'border-radius:12px;' +
        'background:#fff;' +
        'color:#16313d;' +
        'box-shadow:0 8px 18px -16px rgba(0,0,0,0.65);' +
        'min-width:220px;' +
        'font-size:12px;'
    });
  });

  demandNodes.forEach((node, index) => {
    const color = pickNodeColor(node.kind);
    positionedNodes.push({
      id: node.id,
      data: {
        label: node.label
      },
      position: {
        x: xDemand,
        y: 24 + index * yStep
      },
      style:
        `border:1px solid ${color};` +
        'border-radius:12px;' +
        'background:#fff;' +
        'color:#16313d;' +
        'box-shadow:0 8px 18px -16px rgba(0,0,0,0.65);' +
        'min-width:220px;' +
        'font-size:12px;'
    });
  });

  const drawnEdges: Edge[] = edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    animated: edge.flowPerMin > 0.9,
    label: `${edge.flowPerMin.toFixed(2)}/min`,
    style: 'stroke-width:1.6;stroke:#2f4a53;',
    labelStyle: 'font-size:11px;fill:#1f353f;'
  }));

  return {
    nodes: positionedNodes,
    edges: drawnEdges
  };
}
