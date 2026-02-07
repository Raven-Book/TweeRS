import { useState } from 'react';

interface Step {
  code: string;
  description: string;
  array: string[];
  highlight?: number;
  added?: number;
  removed?: number;
}

interface Props {
  variable: string;
  steps: Step[];
}

const HIT = '#6366f1';
const HIT_BG = '#eef2ff';
const ADD_FG = '#16a34a';
const ADD_BG = '#dcfce7';
const DEL_FG = '#dc2626';
const DEL_BG = '#fee2e2';

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
  code: {
    fontFamily: 'monospace',
    fontSize: '14px',
    padding: '8px 12px',
    borderRadius: '6px',
    background: '#fff',
    border: '1px solid #e5e5e5',
    color: '#1a1a1a',
    fontWeight: 600,
  },
  label: {
    fontFamily: 'monospace',
    fontSize: '14px',
    fontWeight: 600,
    color: HIT,
  },
  cells: {
    display: 'flex',
    gap: '8px',
    flexWrap: 'wrap' as const,
    minHeight: '70px',
    alignItems: 'flex-start',
  },
  empty: {
    fontSize: '13px',
    color: '#999',
    padding: '20px 0',
  },
  cellWrapper: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    gap: '4px',
  },
  cell: {
    minWidth: '56px',
    height: '44px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    border: '2px solid',
    borderRadius: '6px',
    fontFamily: 'monospace',
    fontSize: '14px',
    padding: '0 10px',
  },
  index: {
    fontFamily: 'monospace',
    fontSize: '11px',
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

export function ArrayVisualizer({ variable, steps }: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  return (
    <div style={styles.container}>
      {/* code */}
      <div style={styles.code}>
        <code>{step.code}</code>
      </div>

      {/* array label */}
      <div style={styles.label}>{variable}</div>

      {/* cells */}
      <div style={styles.cells}>
        {step.array.length === 0 ? (
          <div style={styles.empty}>（空数组）</div>
        ) : (
          step.array.map((val, i) => {
            const isHighlight = step.highlight === i;
            const isAdded = step.added === i;
            const isRemoved = step.removed === i;

            let borderColor = '#d4d4d4';
            let bg = '#fff';
            let color = '#1a1a1a';
            let textDeco: string | undefined;

            if (isRemoved) {
              borderColor = DEL_FG;
              bg = DEL_BG;
              color = DEL_FG;
              textDeco = 'line-through';
            } else if (isAdded) {
              borderColor = ADD_FG;
              bg = ADD_BG;
              color = ADD_FG;
            } else if (isHighlight) {
              borderColor = HIT;
              bg = HIT_BG;
              color = '#3730a3';
            }

            return (
              <div key={i} style={styles.cellWrapper}>
                <div
                  style={{
                    ...styles.cell,
                    borderColor,
                    background: bg,
                    color,
                    fontWeight: isHighlight || isAdded || isRemoved ? 700 : 500,
                    textDecoration: textDeco,
                    transition: 'all 0.25s ease',
                  }}
                >
                  {val}
                </div>
                <div
                  style={{
                    ...styles.index,
                    color: isHighlight || isAdded || isRemoved ? borderColor : '#999',
                    fontWeight: isHighlight || isAdded || isRemoved ? 700 : 400,
                  }}
                >
                  {i}
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* description */}
      <div style={styles.description}>{step.description}</div>

      {/* controls */}
      <div style={styles.controls}>
        <button
          onClick={() => setCurrent((c) => Math.max(0, c - 1))}
          disabled={current === 0}
          style={styles.btn}
        >
          上一步
        </button>
        <span style={styles.progress}>
          {current + 1} / {steps.length}
        </span>
        <button
          onClick={() => setCurrent((c) => Math.min(steps.length - 1, c + 1))}
          disabled={current === steps.length - 1}
          style={styles.btn}
        >
          下一步
        </button>
      </div>
    </div>
  );
}
