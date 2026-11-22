export type LocationNamespace = "files" | "registry";
export type LocationTone = "created" | "modified" | "deleted" | "neutral";
export type LocationTreeNodeKind = "container" | "value";

export interface LocationRow {
  readonly path: string;
  readonly valueName: string | null;
  readonly label: string;
  readonly tone: LocationTone;
  readonly note: string;
}

export interface LocationTreeNode {
  readonly label: string;
  readonly identity: string;
  readonly kind: LocationTreeNodeKind;
  readonly row: LocationRow | null;
  readonly children: readonly LocationTreeNode[];
}

interface NodeUnderConstruction {
  readonly label: string;
  readonly identity: string;
  readonly kind: LocationTreeNodeKind;
  row: LocationRow | null;
  readonly children: Map<string, NodeUnderConstruction>;
}

const PATH_SEPARATOR = "\\";
const DEFAULT_VALUE_LABEL = "(Default)";

export function buildLocationTree(rows: readonly LocationRow[]): LocationTreeNode[] {
  const root = createNode("", "", "container");
  for (const row of rows) {
    insertRow(root, row);
  }
  return finishChildren(root);
}

export function rowsIn(nodes: readonly LocationTreeNode[]): LocationTreeNode[] {
  return nodes.flatMap((node) => [
    ...(node.row === null ? [] : [node]),
    ...rowsIn(node.children),
  ]);
}

function insertRow(root: NodeUnderConstruction, row: LocationRow): void {
  let container = root;
  for (const segment of row.path.split(PATH_SEPARATOR)) {
    container = childOf(container, segment, "container");
  }
  if (row.valueName === null) {
    container.row = row;
    return;
  }
  const valueLabel = row.valueName === "" ? DEFAULT_VALUE_LABEL : row.valueName;
  childOf(container, valueLabel, "value").row = row;
}

function childOf(
  parent: NodeUnderConstruction,
  label: string,
  kind: LocationTreeNodeKind,
): NodeUnderConstruction {
  const key = `${kind}:${label.toLowerCase()}`;
  const existingChild = parent.children.get(key);
  if (existingChild !== undefined) {
    return existingChild;
  }
  const child = createNode(label, `${parent.identity}${PATH_SEPARATOR}${key}`, kind);
  parent.children.set(key, child);
  return child;
}

function createNode(
  label: string,
  identity: string,
  kind: LocationTreeNodeKind,
): NodeUnderConstruction {
  return { label, identity, kind, row: null, children: new Map() };
}

function finishChildren(parent: NodeUnderConstruction): LocationTreeNode[] {
  return [...parent.children.values()].map(finishNode).sort(compareNodes);
}

function finishNode(node: NodeUnderConstruction): LocationTreeNode {
  let lastNodeOfChain = node;
  let label = node.label;
  let onlyChild = onlyFolderChildOf(lastNodeOfChain);
  while (onlyChild !== null) {
    label = `${label}${PATH_SEPARATOR}${onlyChild.label}`;
    lastNodeOfChain = onlyChild;
    onlyChild = onlyFolderChildOf(lastNodeOfChain);
  }
  return {
    label,
    identity: lastNodeOfChain.identity,
    kind: lastNodeOfChain.kind,
    row: lastNodeOfChain.row,
    children: finishChildren(lastNodeOfChain),
  };
}

function onlyFolderChildOf(node: NodeUnderConstruction): NodeUnderConstruction | null {
  if (node.row !== null || node.children.size !== 1) {
    return null;
  }
  const [onlyChild] = node.children.values();
  const isMergeableFolder =
    onlyChild !== undefined && onlyChild.kind === "container" && onlyChild.children.size > 0;
  return isMergeableFolder ? onlyChild : null;
}

function compareNodes(first: LocationTreeNode, second: LocationTreeNode): number {
  const firstIsBranch = first.children.length > 0;
  const secondIsBranch = second.children.length > 0;
  if (firstIsBranch !== secondIsBranch) {
    return firstIsBranch ? -1 : 1;
  }
  return first.label.localeCompare(second.label, undefined, { sensitivity: "base" });
}
