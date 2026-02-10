import { useState } from 'react';

interface Mapping {
  placeholder: string;
  value: string;
}

interface Step {
  call: string;
  contents?: string;
  mappings: Mapping[];
  result: string[];
  description: string;
}

interface Props {
  name: string;
  isContainer?: boolean;
  definition: string[];
  steps: Step[];
}

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
  sectionLabel: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#999',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
  },
  defBlock: {
    fontFamily: 'monospace',
    fontSize: '14px',
    background: '#fff',
    border: '1px solid #e5e5e5',
    borderRadius: '6px',
    padding: '12px',
    lineHeight: 1.7,
  },
  defLine: {
    color: '#555',
  },
  defBracket: {
    color: '#999',
  },
  placeholder: {
    background: '#eef2ff',
    color: '#6366f1',
    padding: '1px 4px',
    borderRadius: '3px',
    fontWeight: 600,
  },
  columns: {
    display: 'flex',
    gap: '12px',
  },
  column: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  callBlock: {
    fontFamily: 'monospace',
    fontSize: '14px',
    background: '#fff',
    border: '2px solid #6366f1',
    borderRadius: '6px',
    padding: '12px',
    lineHeight: 1.7,
    color: '#6366f1',
    fontWeight: 600,
  },
  resultBlock: {
    fontFamily: 'monospace',
    fontSize: '14px',
    background: '#f0fdf4',
    border: '2px solid #16a34a',
    borderRadius: '6px',
    padding: '12px',
    lineHeight: 1.7,
    color: '#15803d',
  },
  mappingList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
    padding: '0 4px',
  },
  mappingItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '13px',
    fontFamily: 'monospace',
  },
  mappingPlaceholder: {
    color: '#6366f1',
    fontWeight: 600,
  },
  mappingArrow: {
    color: '#999',
  },
  mappingValue: {
    color: '#16a34a',
    fontWeight: 600,
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

function highlightPlaceholders(
  line: string,
  mappings: Mapping[],
): React.ReactNode[] {
  if (mappings.length === 0) return [line];

  const parts: React.ReactNode[] = [];
  let remaining = line;
  let key = 0;

  while (remaining.length > 0) {
    let earliest = -1;
    let earliestMapping: Mapping | null = null;

    for (const m of mappings) {
      const idx = remaining.indexOf(m.placeholder);
      if (idx !== -1 && (earliest === -1 || idx < earliest)) {
        earliest = idx;
        earliestMapping = m;
      }
    }

    if (earliest === -1 || !earliestMapping) {
      parts.push(remaining);
      break;
    }

    if (earliest > 0) {
      parts.push(remaining.slice(0, earliest));
    }
    parts.push(
      <span key={key++} style={styles.placeholder}>
        {earliestMapping.placeholder}
      </span>,
    );
    remaining = remaining.slice(earliest + earliestMapping.placeholder.length);
  }

  return parts;
}

export function WidgetVisualizer({
  name,
  isContainer,
  definition,
  steps,
}: Props) {
  const [current, setCurrent] = useState(0);
  const step = steps[current];

  const openTag = isContainer
    ? `<<widget "${name}" container>>`
    : `<<widget "${name}">>`;
  const closeTag = `<</widget>>`;

  return (
    <div style={styles.container}>
      {/* Definition */}
      <div style={styles.sectionLabel}>定义</div>
      <div style={styles.defBlock}>
        <div style={styles.defBracket}>{openTag}</div>
        {definition.map((line, i) => (
          <div key={i} style={{ ...styles.defLine, paddingLeft: '16px' }}>
            {highlightPlaceholders(line, step.mappings)}
          </div>
        ))}
        <div style={styles.defBracket}>{closeTag}</div>
      </div>

      {/* Call → Result */}
      <div style={styles.columns}>
        <div style={styles.column}>
          <div style={styles.sectionLabel}>调用</div>
          <div style={styles.callBlock}>
            <div>{step.call}</div>
            {step.contents != null && (
              <div style={{ color: '#444', fontWeight: 400 }}>
                {step.contents}
              </div>
            )}
            {isContainer && (
              <div style={{ color: '#6366f1' }}>{`<</${name}>>`}</div>
            )}
          </div>
          {step.mappings.length > 0 && (
            <div style={styles.mappingList}>
              {step.mappings.map((m, i) => (
                <div key={i} style={styles.mappingItem}>
                  <span style={styles.mappingPlaceholder}>{m.placeholder}</span>
                  <span style={styles.mappingArrow}>←</span>
                  <span style={styles.mappingValue}>{m.value}</span>
                </div>
              ))}
            </div>
          )}
        </div>
        <div style={styles.column}>
          <div style={styles.sectionLabel}>展开结果</div>
          <div style={styles.resultBlock}>
            {step.result.map((line, i) => (
              <div key={i}>{line}</div>
            ))}
          </div>
        </div>
      </div>

      <div style={styles.description}>{step.description}</div>

      {/* Controls */}
      {steps.length > 1 && (
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
            onClick={() =>
              setCurrent(Math.min(steps.length - 1, current + 1))
            }
            disabled={current === steps.length - 1}
            style={styles.btn}
          >
            下一步
          </button>
        </div>
      )}
    </div>
  );
}
