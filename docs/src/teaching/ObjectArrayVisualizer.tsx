import { useState } from 'react';

interface Entry {
  key: string;
  value: string;
}

interface Item {
  entries: Entry[];
}

interface Step {
  code: string;
  description: string;
  items: Item[];
  highlightIndex?: number;
  highlightKey?: string;
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
  cards: {
    display: 'flex',
    gap: '10px',
    flexWrap: 'wrap' as const,
    alignItems: 'flex-start',
  },
  cardWrapper: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    gap: '4px',
  },
  card: {
    borderRadius: '6px',
    border: '2px solid',
    overflow: 'hidden' as const,
    fontFamily: 'monospace',
    fontSize: '13px',
    transition: 'all 0.25s ease',
  },
  row: {
    display: 'flex',
  },
  key: {
    padding: '4px 10px',
    fontWeight: 600,
    borderRight: '1px solid',
    minWidth: '70px',
  },
  val: {
    padding: '4px 10px',
    minWidth: '50px',
  },
  index: {
    fontFamily: 'monospace',
    fontSize: '11px',
    color: '#999',
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

export function ObjectArrayVisualizer({ variable, steps }: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  return (
    <div style={styles.container}>
      <div style={styles.code}>
        <code>{step.code}</code>
      </div>

      <div style={styles.label}>{variable}</div>

      <div style={styles.cards}>
        {step.items.map((item, i) => {
          const isHighlight = step.highlightIndex === i;
          const isAdded = step.added === i;
          const isRemoved = step.removed === i;
          const active = isHighlight || isAdded || isRemoved;

          let borderColor = '#d4d4d4';
          let bg = '#fff';
          if (isRemoved) {
            borderColor = DEL_FG;
            bg = DEL_BG;
          } else if (isAdded) {
            borderColor = ADD_FG;
            bg = ADD_BG;
          } else if (isHighlight) {
            borderColor = HIT;
            bg = HIT_BG;
          }

          return (
            <div key={i} style={styles.cardWrapper}>
              <div style={{ ...styles.card, borderColor }}>
                {item.entries.map((entry, j) => {
                  const keyHit = isHighlight && step.highlightKey === entry.key;
                  return (
                    <div
                      key={entry.key}
                      style={{
                        ...styles.row,
                        borderTop: j > 0 ? `1px solid ${active ? borderColor : '#e5e5e5'}` : undefined,
                        background: keyHit ? HIT_BG : bg,
                      }}
                    >
                      <div
                        style={{
                          ...styles.key,
                          borderRightColor: active ? borderColor : '#e5e5e5',
                          color: active ? '#3730a3' : '#525252',
                        }}
                      >
                        {entry.key}
                      </div>
                      <div
                        style={{
                          ...styles.val,
                          color: keyHit ? '#3730a3' : isRemoved ? DEL_FG : '#1a1a1a',
                          fontWeight: keyHit ? 700 : 400,
                          textDecoration: isRemoved ? 'line-through' : undefined,
                        }}
                      >
                        {entry.value}
                      </div>
                    </div>
                  );
                })}
              </div>
              <div
                style={{
                  ...styles.index,
                  color: active ? borderColor : '#999',
                  fontWeight: active ? 700 : 400,
                }}
              >
                {i}
              </div>
            </div>
          );
        })}
      </div>

      <div style={styles.description}>{step.description}</div>

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
