import dagre from '@dagrejs/dagre';
import type { Edge, Node } from '@xyflow/svelte';
import {
  NODE_WIDTH,
  SCC_CLUSTER_PADDING,
  SCC_MIN_SIZE,
  type LayoutNode,
  type SCCResult
} from './types';

interface CenterPoint {
  x: number;
  y: number;
}

interface SCCCluster {
  id: string;
  nodeIds: string[];
  width: number;
  height: number;
  relativeCenters: Map<string, CenterPoint>;
}

/**
 * 对单个 SCC 内部做 TB 布局，返回内部节点相对 cluster 左上角的中心点。
 */
function layoutSccInternals(
  nodeIds: string[],
  nodeMap: Map<string, LayoutNode>,
  edges: Edge[]
): SCCCluster {
  const graph = new dagre.graphlib.Graph({ multigraph: true });
  graph.setDefaultEdgeLabel(() => ({}));
  graph.setGraph({
    rankdir: 'TB',
    ranksep: 72,
    nodesep: 36,
    marginx: 0,
    marginy: 0
  });

  const nodeIdSet = new Set(nodeIds);
  const layoutNodeIds: string[] = [];

  let maxNodeWidth = 0;
  let maxNodeHeight = 0;
  for (const id of nodeIds) {
    const node = nodeMap.get(id);
    if (!node) {
      continue;
    }

    layoutNodeIds.push(id);
    maxNodeWidth = Math.max(maxNodeWidth, node.width);
    maxNodeHeight = Math.max(maxNodeHeight, node.height);
    graph.setNode(id, {
      width: node.width,
      height: node.height
    });
  }

  for (const edge of edges) {
    if (!nodeIdSet.has(edge.source) || !nodeIdSet.has(edge.target)) {
      continue;
    }

    graph.setEdge(edge.source, edge.target, {}, edge.id);
  }

  dagre.layout(graph);

  let minLeft = Number.POSITIVE_INFINITY;
  let maxRight = Number.NEGATIVE_INFINITY;
  let minTop = Number.POSITIVE_INFINITY;
  let maxBottom = Number.NEGATIVE_INFINITY;
  const nodeCenterMap = new Map<string, CenterPoint>();

  for (const id of layoutNodeIds) {
    const node = nodeMap.get(id);
    const point = graph.node(id) as CenterPoint | undefined;
    if (!node || !point) {
      continue;
    }

    minLeft = Math.min(minLeft, point.x - node.width / 2);
    maxRight = Math.max(maxRight, point.x + node.width / 2);
    minTop = Math.min(minTop, point.y - node.height / 2);
    maxBottom = Math.max(maxBottom, point.y + node.height / 2);
    nodeCenterMap.set(id, point);
  }

  const clusterId = '__internal_scc__';
  if (
    layoutNodeIds.length === 0 ||
    !Number.isFinite(minLeft) ||
    !Number.isFinite(maxRight) ||
    !Number.isFinite(minTop) ||
    !Number.isFinite(maxBottom)
  ) {
    return {
      id: clusterId,
      nodeIds: layoutNodeIds,
      width: Math.max(maxNodeWidth + SCC_CLUSTER_PADDING * 2, NODE_WIDTH + SCC_CLUSTER_PADDING * 2),
      height: maxNodeHeight + SCC_CLUSTER_PADDING * 2,
      relativeCenters: new Map()
    };
  }

  const innerWidth = maxRight - minLeft;
  const innerHeight = maxBottom - minTop;
  const contentWidth = innerWidth + SCC_CLUSTER_PADDING * 2;
  const contentHeight = innerHeight + SCC_CLUSTER_PADDING * 2;

  const width = Math.max(contentWidth, NODE_WIDTH + SCC_CLUSTER_PADDING * 2);
  const height = Math.max(contentHeight, maxNodeHeight + SCC_CLUSTER_PADDING * 2);
  const contentOffsetX = (width - contentWidth) / 2;
  const contentOffsetY = (height - contentHeight) / 2;

  const relativeCenters = new Map<string, CenterPoint>();
  for (const [id, point] of nodeCenterMap.entries()) {
    relativeCenters.set(id, {
      x: contentOffsetX + SCC_CLUSTER_PADDING + (point.x - minLeft),
      y: contentOffsetY + SCC_CLUSTER_PADDING + (point.y - minTop)
    });
  }

  return {
    id: clusterId,
    nodeIds: layoutNodeIds,
    width,
    height,
    relativeCenters
  };
}

export function layoutNodesWithDagre(
  nodes: Node[],
  edges: Edge[],
  layoutNodes: LayoutNode[],
  sccResult?: SCCResult
): Node[] {
  const graph = new dagre.graphlib.Graph({ multigraph: true });
  graph.setDefaultEdgeLabel(() => ({}));
  graph.setGraph({
    rankdir: 'LR',
    ranksep: 140,
    nodesep: 36,
    marginx: 24,
    marginy: 24
  });

  const layoutNodeMap = new Map<string, LayoutNode>();
  for (const layoutNode of layoutNodes) {
    layoutNodeMap.set(layoutNode.id, layoutNode);
  }

  // 识别需要作为子图的 SCC（大小 >= SCC_MIN_SIZE）
  const clusters: SCCCluster[] = [];
  if (sccResult) {
    for (let i = 0; i < sccResult.components.length; i++) {
      const component = sccResult.components[i];
      if (component.length < SCC_MIN_SIZE) {
        continue;
      }

      const clusterId = `__scc_cluster_${i}`;
      const internalLayout = layoutSccInternals(component, layoutNodeMap, edges);
      clusters.push({
        id: clusterId,
        nodeIds: internalLayout.nodeIds,
        width: internalLayout.width,
        height: internalLayout.height,
        relativeCenters: internalLayout.relativeCenters
      });
    }
  }

  const nodeToClusterId = new Map<string, string>();
  for (const cluster of clusters) {
    for (const nodeId of cluster.nodeIds) {
      nodeToClusterId.set(nodeId, cluster.id);
    }
  }

  // 先创建外层节点（包括 SCC 超节点）
  for (const cluster of clusters) {
    graph.setNode(cluster.id, {
      width: cluster.width,
      height: cluster.height
    });
  }

  for (const node of nodes) {
    if (nodeToClusterId.has(node.id)) {
      continue;
    }

    const layoutNode = layoutNodeMap.get(node.id);
    if (!layoutNode) {
      continue;
    }

    graph.setNode(node.id, {
      width: layoutNode.width,
      height: layoutNode.height
    });
  }

  // SCC 内部边不参与外层布局，跨 SCC 的边映射到超节点。
  for (const edge of edges) {
    const source = nodeToClusterId.get(edge.source) ?? edge.source;
    const target = nodeToClusterId.get(edge.target) ?? edge.target;
    if (source === target) {
      continue;
    }

    graph.setEdge(source, target, {}, edge.id);
  }

  dagre.layout(graph);

  const centerByNodeId = new Map<string, CenterPoint>();
  for (const node of nodes) {
    if (nodeToClusterId.has(node.id)) {
      continue;
    }

    const point = graph.node(node.id) as CenterPoint | undefined;
    if (!point) {
      continue;
    }

    centerByNodeId.set(node.id, point);
  }

  for (const cluster of clusters) {
    const clusterPoint = graph.node(cluster.id) as CenterPoint | undefined;
    if (!clusterPoint) {
      continue;
    }

    const clusterLeft = clusterPoint.x - cluster.width / 2;
    const clusterTop = clusterPoint.y - cluster.height / 2;

    for (const [nodeId, relativeCenter] of cluster.relativeCenters.entries()) {
      centerByNodeId.set(nodeId, {
        x: clusterLeft + relativeCenter.x,
        y: clusterTop + relativeCenter.y
      });
    }
  }

  return nodes.map((node) => {
    const point = centerByNodeId.get(node.id);
    const layoutNode = layoutNodeMap.get(node.id);
    if (!point || !layoutNode) {
      return node;
    }

    return {
      ...node,
      position: {
        x: point.x - layoutNode.xOffset,
        y: point.y - layoutNode.yOffset
      }
    };
  });
}
