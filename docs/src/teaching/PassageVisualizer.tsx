import { useState, useCallback, useMemo } from 'react';
import {
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  Handle,
  Position,
  type NodeProps,
  type Node as RFNode,
  type Edge as RFEdge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

interface PassageNodeDef {
  id: string;
  title: string;
  content: string[];
  x: number;
  y: number;
  special?: boolean;
}

interface EdgeDef {
  id: string;
  source: string;
  target: string;
  label?: string;
}

interface Step {
  visibleNodes: string[];
  visibleEdges: string[];
  activeNode: string;
  description: string;
}

interface Props {
  nodes: PassageNodeDef[];
  edges: EdgeDef[];
  steps: Step[];
}

const ACTIVE = '#6366f1';
const ACTIVE_BG = '#eef2ff';
const SPECIAL_BORDER = '#a3a3a3';
const SPECIAL_BG = '#f5f5f5';
const NEUTRAL_BORDER = '#d4d4d4';
const NEUTRAL_BG = '#fff';

function PassageNode({ data }: NodeProps) {
  const active = data.active as boolean;
  const special = data.special as boolean;
  const title = data.title as string;
  const content = data.content as string[];

  const borderColor = active ? ACTIVE : special ? SPECIAL_BORDER : NEUTRAL_BORDER;
  const bg = active ? ACTIVE_BG : special ? SPECIAL_BG : NEUTRAL_BG;

  return (
    <div
      style={{
        minWidth: '120px',
        maxWidth: '180px',
        borderRadius: '8px',
        border: `2px ${special ? 'dashed' : 'solid'} ${borderColor}`,
        background: bg,
        fontSize: '12px',
        overflow: 'hidden',
        transition: 'all 0.25s ease',
      }}
    >
      <div
        style={{
          padding: '4px 10px',
          background: active ? ACTIVE : special ? '#d4d4d4' : '#e5e5e5',
          color: active ? '#fff' : '#333',
          fontWeight: 600,
          fontSize: '12px',
          fontFamily: 'monospace',
        }}
      >
        :: {title}
      </div>
      {content.length > 0 && (
        <div
          style={{
            padding: '6px 10px',
            color: '#555',
            lineHeight: '1.5',
            whiteSpace: 'pre-wrap',
          }}
        >
          {content.map((line, i) => (
            <div key={i}>{line}</div>
          ))}
        </div>
      )}
      <Handle
        type="target"
        position={Position.Left}
        id="left-target"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="right-source"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="target"
        position={Position.Top}
        id="top-target"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="bottom-source"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
    </div>
  );
}

const nodeTypes = { passage: PassageNode };

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
    padding: '16px',
    border: '1px solid #e5e5e5',
    borderRadius: '8px',
    marginBlock: '16px',
    background: '#fafafa',
  },
  flowWrapper: {
    height: '340px',
    borderRadius: '6px',
    overflow: 'hidden' as const,
    position: 'relative' as const,
  },
  description: {
    fontSize: '13px',
    color: '#666',
  },
  controls: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '12px',
  },
  btn: {
    padding: '4px 12px',
    border: '1px solid #e5e5e5',
    borderRadius: '4px',
    background: '#fff',
    cursor: 'pointer',
    fontSize: '13px',
  },
  progress: {
    fontSize: '13px',
    color: '#999',
  },
  zoomControls: {
    position: 'absolute' as const,
    top: '8px',
    right: '8px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
    zIndex: 5,
  },
  zoomBtn: {
    width: '28px',
    height: '28px',
    border: '1px solid #e5e5e5',
    borderRadius: '4px',
    background: '#fff',
    cursor: 'pointer',
    fontSize: '16px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: '#555',
  },
};

function ZoomControls() {
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  return (
    <div style={styles.zoomControls}>
      <button onClick={() => zoomIn()} style={styles.zoomBtn} title="放大">+</button>
      <button onClick={() => zoomOut()} style={styles.zoomBtn} title="缩小">−</button>
      <button onClick={() => fitView()} style={styles.zoomBtn} title="适应">⊡</button>
    </div>
  );
}

function PassageVisualizerInner({ nodes, edges, steps }: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  const rfNodes: RFNode[] = useMemo(
    () =>
      nodes
        .filter((n) => step.visibleNodes.includes(n.id))
        .map((n) => ({
          id: n.id,
          type: 'passage',
          position: { x: n.x, y: n.y },
          data: {
            title: n.title,
            content: n.content,
            special: n.special ?? false,
            active: n.id === step.activeNode,
          },
        })),
    [nodes, step],
  );

  const rfEdges: RFEdge[] = useMemo(
    () =>
      edges
        .filter((e) => step.visibleEdges.includes(e.id))
        .map((e) => ({
          id: e.id,
          source: e.source,
          target: e.target,
          sourceHandle: 'right-source',
          targetHandle: 'left-target',
          label: e.label,
          animated: e.target === step.activeNode,
          style: {
            stroke: e.target === step.activeNode ? ACTIVE : '#b0b0b0',
            strokeWidth: e.target === step.activeNode ? 2 : 1.5,
          },
          labelStyle: {
            fontSize: '11px',
            fontWeight: 600,
            fill: e.target === step.activeNode ? ACTIVE : '#888',
          },
        })),
    [edges, step],
  );

  const onPrev = useCallback(() => setCurrent((c) => Math.max(0, c - 1)), []);
  const onNext = useCallback(
    () => setCurrent((c) => Math.min(steps.length - 1, c + 1)),
    [steps.length],
  );

  return (
    <div style={styles.container}>
      <div style={styles.flowWrapper}>
        <ReactFlow
          nodes={rfNodes}
          edges={rfEdges}
          nodeTypes={nodeTypes}
          fitView
          nodesConnectable={false}
          elementsSelectable={false}
          zoomOnScroll={false}
          zoomOnPinch={false}
          zoomOnDoubleClick={false}
          preventScrolling={false}
          proOptions={{ hideAttribution: true }}
        />
        <ZoomControls />
      </div>
      <div style={styles.description}>{step.description}</div>
      <div style={styles.controls}>
        <button onClick={onPrev} disabled={current === 0} style={styles.btn}>
          上一步
        </button>
        <span style={styles.progress}>
          {current + 1} / {steps.length}
        </span>
        <button
          onClick={onNext}
          disabled={current === steps.length - 1}
          style={styles.btn}
        >
          下一步
        </button>
      </div>
    </div>
  );
}

export function PassageVisualizer(props: Props) {
  return (
    <ReactFlowProvider>
      <PassageVisualizerInner {...props} />
    </ReactFlowProvider>
  );
}