import { describe, expect, it } from 'vitest';
import { buildFlowGraph } from './graph';
import type { LogisticsGraphDto } from './types';

const graphFixture: LogisticsGraphDto = {
  items: [
    { itemKey: 'A', itemName: 'Item A', edgeCount: 1, nodeCount: 2, totalFlowPerMin: 3.2 },
    { itemKey: 'B', itemName: 'Item B', edgeCount: 1, nodeCount: 2, totalFlowPerMin: 1.1 }
  ],
  nodes: [
    { id: 'n1', itemKey: 'A', itemName: 'Item A', kind: 'external_supply', label: 'Supply A' },
    { id: 'n2', itemKey: 'A', itemName: 'Item A', kind: 'outpost_sale', label: 'Sale A' },
    { id: 'n3', itemKey: 'B', itemName: 'Item B', kind: 'external_supply', label: 'Supply B' },
    { id: 'n4', itemKey: 'B', itemName: 'Item B', kind: 'outpost_sale', label: 'Sale B' }
  ],
  edges: [
    {
      id: 'e1',
      itemKey: 'A',
      itemName: 'Item A',
      source: 'n1',
      target: 'n2',
      flowPerMin: 3.2
    },
    {
      id: 'e2',
      itemKey: 'B',
      itemName: 'Item B',
      source: 'n3',
      target: 'n4',
      flowPerMin: 1.1
    }
  ]
};

describe('buildFlowGraph', () => {
  it('returns all nodes and edges for all filter', () => {
    const flow = buildFlowGraph(graphFixture, 'all');

    expect(flow.edges).toHaveLength(2);
    expect(flow.nodes).toHaveLength(4);
    expect(flow.edges[0]?.label).toMatch('/min');
  });

  it('filters graph by item key', () => {
    const flow = buildFlowGraph(graphFixture, 'A');

    expect(flow.edges).toHaveLength(1);
    expect(flow.edges[0]?.id).toBe('e1');
    expect(flow.nodes.map((node) => node.id).sort()).toEqual(['n1', 'n2']);
  });
});
