export type TreeNodeKind = 'initial' | 'harmony' | 'note';

export interface TreeNodeInfo {
  id: number;
  parent: number | null;
  measureIndex: number;
  voiceIndex: number | null;
  kind: TreeNodeKind;
  nStep: number | null;
  cost: number | null;
  thisCost: number | null;
  debug: string;
  isGoal: boolean;
  nExpanded: number;
  isStart: boolean;
  onSolutionPath: boolean;
  bestPriority: number | null;
  maxDepth: number;
  subtreeSize: number;
}

export interface TreeNode extends TreeNodeInfo {
  children: TreeNodeInfo[];
}

export interface TreeMeta {
  nodeCount: number;
  start: number;
  goal: number | null;
  solutionPath: number[];
  targetMeasures: number;
}

async function getJson<T>(url: string): Promise<T> {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`GET ${url} -> ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export function fetchTreeMeta(): Promise<TreeMeta> {
  return getJson<TreeMeta>('/tree');
}

export function fetchTreeNode(id: number): Promise<TreeNode> {
  return getJson<TreeNode>(`/tree/node/${id}`);
}

export function fetchNodeScore(id: number): Promise<{ mxl: string }> {
  return getJson<{ mxl: string }>(`/tree/score/${id}`);
}
