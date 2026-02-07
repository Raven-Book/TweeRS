import { useState } from 'react';

interface Entry {
  key: string;
  value: string;
}

interface Step {
  code: string;
  description: string;
  entries: Entry[];
  highlight?: string;
  changed?: string;
  prevValue?: string;
}

interface Props {
  variable: string;
  steps: Step[];
}

const HIT = '#6366f1';
const HIT_BG = '#eef2ff';
const CHG_FG = '#16a34a';
const CHG_BG = '#dcfce7';
const OLD_FG = '#dc2626';

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
  table: {
    borderCollapse: 'collapse' as const,
    fontFamily: 'monospace',
    fontSize: '14px',
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

export function ObjectVisualizer({ variable, steps }: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  return (
    <div style={styles.container}>
      <div style={styles.code}>
        <code>{step.code}</code>
      </div>

      <div style={styles.label}>{variable}</div>

      <table style={styles.table}>
        <tbody>
          {step.entries.map((entry) => {
            const isHighlight = step.highlight === entry.key;
            const isChanged = step.changed === entry.key;
            const active = isHighlight || isChanged;

            const rowBg = active ? HIT_BG : '#fff';
            const borderColor = active ? HIT : '#e5e5e5';

            return (
              <tr key={entry.key}>
                <td
                  style={{
                    padding: '8px 14px',
                    border: `1.5px solid ${borderColor}`,
                    fontWeight: 600,
                    color: active ? '#3730a3' : '#525252',
                    background: rowBg,
                    transition: 'all 0.25s ease',
                  }}
                >
                  {entry.key}
                </td>
                <td
                  style={{
                    padding: '8px 14px',
                    border: `1.5px solid ${borderColor}`,
                    background: rowBg,
                    transition: 'all 0.25s ease',
                  }}
                >
                  {isChanged && step.prevValue != null && (
                    <span
                      style={{
                        color: OLD_FG,
                        textDecoration: 'line-through',
                        marginRight: '8px',
                        opacity: 0.7,
                      }}
                    >
                      {step.prevValue}
                    </span>
                  )}
                  <span
                    style={{
                      color: isChanged ? CHG_FG : active ? '#3730a3' : '#1a1a1a',
                      fontWeight: active ? 700 : 500,
                    }}
                  >
                    {entry.value}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>

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
