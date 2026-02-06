import { useState } from 'react';

interface Step {
  code: string;
  variable: string;
  value: string;
  prevValue?: string;
  description: string;
}

interface Props {
  steps: Step[];
  hint?: string;
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '16px',
    padding: '16px',
    border: '1px solid #e5e5e5',
    borderRadius: '8px',
    marginBlock: '16px',
    background: '#fafafa',
  },
  hint: {
    fontSize: '13px',
    color: '#666',
  },
  codePanel: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '2px',
    fontFamily: 'monospace',
    fontSize: '14px',
  },
  codeLine: {
    padding: '4px 8px',
    borderRadius: '4px',
    display: 'flex',
    gap: '8px',
    alignItems: 'center',
    color: '#999',
  },
  codeLineActive: {
    background: '#e8f4fd',
    color: '#1a1a1a',
    fontWeight: 600,
  },
  codeLineDone: {
    color: '#aaa',
    textDecoration: 'line-through' as const,
  },
  lineNum: {
    width: '20px',
    textAlign: 'right' as const,
    fontSize: '12px',
    opacity: 0.5,
  },
  boxPanel: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    gap: '8px',
    padding: '16px 0',
  },
  boxLabel: {
    fontFamily: 'monospace',
    fontSize: '14px',
    fontWeight: 600,
    color: '#6366f1',
  },
  box: {
    position: 'relative' as const,
    width: '120px',
    height: '60px',
    border: '2px solid #6366f1',
    borderRadius: '8px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    background: '#fff',
    overflow: 'hidden' as const,
  },
  value: {
    fontFamily: 'monospace',
    fontSize: '20px',
    fontWeight: 700,
    color: '#1a1a1a',
    transition: 'all 0.3s ease',
  },
  prevValue: {
    position: 'absolute' as const,
    fontFamily: 'monospace',
    fontSize: '16px',
    color: '#ef4444',
    textDecoration: 'line-through' as const,
    top: '4px',
    right: '8px',
    opacity: 0.6,
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
};

export function AssignmentVisualizer({ steps, hint }: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  return (
    <div style={styles.container}>
      {hint && <div style={styles.hint}>{hint}</div>}
      <div style={styles.codePanel}>
        {steps.map((s, i) => (
          <div
            key={i}
            style={{
              ...styles.codeLine,
              ...(i === current ? styles.codeLineActive : {}),
              ...(i < current ? styles.codeLineDone : {}),
            }}
          >
            <span style={styles.lineNum}>{i + 1}</span>
            <code>{s.code}</code>
          </div>
        ))}
      </div>

      <div style={styles.boxPanel}>
        <div style={styles.boxLabel}>{step.variable}</div>
        <div style={styles.box}>
          {step.prevValue !== undefined && (
            <div style={styles.prevValue}>{step.prevValue}</div>
          )}
          <div style={styles.value}>{step.value}</div>
        </div>
        <div style={styles.description}>{step.description}</div>
      </div>

      <div style={styles.controls}>
        <button
          onClick={() => setCurrent(Math.max(0, current - 1))}
          disabled={current === 0}
          style={styles.btn}
        >
          上一步
        </button>
        <span style={styles.progress}>
          {current + 1} / {steps.length}
        </span>
        <button
          onClick={() => setCurrent(Math.min(steps.length - 1, current + 1))}
          disabled={current === steps.length - 1}
          style={styles.btn}
        >
          下一步
        </button>
      </div>
    </div>
  );
}
