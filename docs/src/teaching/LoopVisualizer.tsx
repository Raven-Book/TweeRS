import { useState, useCallback, useMemo } from 'react';
import {
  ReactFlow,
  Handle,
  Position,
  type NodeProps,
  type Node as RFNode,
  type Edge as RFEdge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

interface NodeDef {
  id: string;
  label: string;
  x: number;
  y: number;
}

interface EdgeDef {
  id: string;
  source: string;
  target: string;
  label?: string;
  sourceHandle?: string;
  targetHandle?: string;
}

interface Step {
  nodeId: string;
  label: string;
  variables: Record<string, string>;
}

interface Props {
  nodes: NodeDef[];
  edges: EdgeDef[];
  steps: Step[];
}

const ACTIVE = '#6366f1';
const ACTIVE_BG = '#eef2ff';
const NEUTRAL_BORDER = '#d4d4d4';
const NEUTRAL_BG = '#fff';

function LoopNode({ data }: NodeProps) {
  const active = data.active as boolean;

  return (
    <div
      style={{
        padding: '8px 16px',
        borderRadius: '8px',
        border: `2px solid ${active ? ACTIVE : NEUTRAL_BORDER}`,
        background: active ? ACTIVE_BG : NEUTRAL_BG,
        fontSize: '13px',
        fontFamily: 'monospace',
        fontWeight: active ? 600 : 400,
        color: active ? '#3730a3' : '#525252',
        transition: 'all 0.25s ease',
        whiteSpace: 'nowrap',
      }}
    >
      {/* Default handles (right/left) */}
      <Handle
        type="target"
        position={Position.Left}
        id="default-target"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="default-source"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      {/* Extra handles for loop-back edges */}
      <Handle
        type="source"
        position={Position.Left}
        id="left-src"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="target"
        position={Position.Top}
        id="top"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="source"
        position={Position.Top}
        id="top-src"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="bottom-src"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      <Handle
        type="target"
        position={Position.Bottom}
        id="bottom-target"
        style={{ opacity: 0, width: 1, height: 1 }}
      />
      {data.label as string}
    </div>
  );
}

const nodeTypes = { loop: LoopNode };

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
    height: '160px',
    borderRadius: '6px',
    overflow: 'hidden' as const,
  },
  description: {
    fontSize: '13px',
    color: '#666',
  },
  varsPanel: {
    display: 'flex',
    gap: '12px',
    flexWrap: 'wrap' as const,
    fontFamily: 'monospace',
    fontSize: '13px',
  },
  varItem: {
    padding: '4px 10px',
    borderRadius: '4px',
    background: '#eef2ff',
    color: '#3730a3',
    fontWeight: 600,
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
};

export function LoopVisualizer({ nodes, edges, steps }: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  const rfNodes: RFNode[] = useMemo(
    () =>
      nodes.map((n) => ({
        id: n.id,
        type: 'loop',
        position: { x: n.x, y: n.y },
        data: { label: n.label, active: n.id === step.nodeId },
      })),
    [nodes, step.nodeId],
  );

  const rfEdges: RFEdge[] = useMemo(
    () =>
      edges.map((e) => {
        const active = e.target === step.nodeId;
        return {
          id: e.id,
          source: e.source,
          target: e.target,
          sourceHandle: e.sourceHandle ?? 'default-source',
          targetHandle: e.targetHandle ?? 'default-target',
          label: e.label,
          animated: active,
          style: {
            stroke: active ? ACTIVE : '#b0b0b0',
            strokeWidth: active ? 2 : 1.5,
            transition: 'stroke 0.25s ease',
          },
          labelStyle: {
            fontSize: '11px',
            fontWeight: 600,
            fill: active ? ACTIVE : '#888',
          },
        };
      }),
    [edges, step.nodeId],
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
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable={false}
          panOnDrag={false}
          zoomOnScroll={false}
          zoomOnPinch={false}
          zoomOnDoubleClick={false}
          preventScrolling={false}
          proOptions={{ hideAttribution: true }}
        />
      </div>

      <div style={styles.description}>{step.label}</div>

      <div style={styles.varsPanel}>
        {Object.entries(step.variables).map(([k, v]) => (
          <span key={k} style={styles.varItem}>
            {k} = {v}
          </span>
        ))}
      </div>

      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          gap: '6px',
        }}
      >
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
    </div>
  );
}
