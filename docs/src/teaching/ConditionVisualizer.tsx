import { useState } from 'react';

interface Branch {
  condition: string;
  result: string;
}

interface Props {
  variable: string;
  min: number;
  max: number;
  initial: number;
  branches: Branch[];
  fallback: string;
}

function evaluate(condition: string, value: number): boolean {
  const match = condition.match(/^(>=?|<=?|==)\s*(-?\d+)$/);
  if (!match) return false;
  const [, op, num] = match;
  const n = Number(num);
  switch (op) {
    case '>': return value > n;
    case '>=': return value >= n;
    case '<': return value < n;
    case '<=': return value <= n;
    case '==': return value === n;
    default: return false;
  }
}

const HIT = '#4f46e5';
const TRUE_BG = '#dcfce7';
const TRUE_FG = '#16a34a';
const FALSE_BG = '#fee2e2';
const FALSE_FG = '#dc2626';

function Badge({ hit }: { hit: boolean }) {
  return (
    <span style={{
      fontSize: '11px',
      fontWeight: 600,
      padding: '2px 8px',
      borderRadius: '4px',
      color: hit ? TRUE_FG : FALSE_FG,
      background: hit ? TRUE_BG : FALSE_BG,
    }}>
      {hit ? 'true' : 'false'}
    </span>
  );
}

function Connector({ dimmed }: { dimmed: boolean }) {
  return (
    <div style={{
      width: '2px',
      height: '12px',
      background: dimmed ? '#e5e5e5' : HIT,
      marginLeft: '24px',
      transition: 'background 0.25s ease',
    }} />
  );
}

function ResultBubble({ text }: { text: string }) {
  return (
    <div style={{
      marginLeft: '24px',
      marginTop: '4px',
      padding: '8px 14px',
      borderRadius: '8px',
      background: '#eef2ff',
      border: `1.5px solid ${HIT}`,
      color: '#3730a3',
      fontSize: '13px',
      fontWeight: 500,
    }} >
      {text}
    </div>
  );
}

export function ConditionVisualizer({
  variable, min, max, initial, branches, fallback,
}: Props) {
  const [value, setValue] = useState(initial);

  let hitIndex = -1;
  for (let i = 0; i < branches.length; i++) {
    if (evaluate(branches[i].condition, value)) {
      hitIndex = i;
      break;
    }
  }

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      padding: '16px',
      border: '1px solid #e5e5e5',
      borderRadius: '8px',
      marginBlock: '16px',
      background: '#fafafa',
    }}>
      {/* slider */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        marginBottom: '16px',
      }}>
        <span style={{
          fontFamily: 'monospace',
          fontSize: '15px',
          fontWeight: 600,
          color: HIT,
          whiteSpace: 'nowrap',
        }}>
          {variable} = {value}
        </span>
        <input
          type="range"
          min={min}
          max={max}
          value={value}
          onChange={(e) => setValue(Number(e.target.value))}
          style={{ flex: 1 }}
        />
      </div>

      {/* flow */}
      {branches.map((b, i) => {
        const isHit = i === hitIndex;
        const skipped = hitIndex >= 0 && i > hitIndex;

        return (
          <div key={i}>
            {i > 0 && <Connector dimmed={skipped} />}
            <div style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '8px 14px',
              borderRadius: '8px',
              border: `1.5px solid ${isHit ? HIT : '#e5e5e5'}`,
              background: isHit ? '#eef2ff' : '#fff',
              opacity: skipped ? 0.35 : 1,
              transition: 'all 0.25s ease',
              fontFamily: 'monospace',
              fontSize: '13px',
            }}>
              <span style={{ flex: 1 }}>
                {i === 0 ? 'if' : 'elseif'} {variable} {b.condition}
              </span>
              <Badge hit={isHit} />
            </div>
            {isHit && <ResultBubble text={b.result} />}
          </div>
        );
      })}

      {/* else fallback */}
      <Connector dimmed={hitIndex >= 0} />
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '8px 14px',
        borderRadius: '8px',
        border: `1.5px solid ${hitIndex < 0 ? HIT : '#e5e5e5'}`,
        background: hitIndex < 0 ? '#eef2ff' : '#fff',
        opacity: hitIndex >= 0 ? 0.35 : 1,
        transition: 'all 0.25s ease',
        fontFamily: 'monospace',
        fontSize: '13px',
      }}>
        <span style={{ flex: 1 }}>else</span>
        {hitIndex < 0 && <Badge hit />}
      </div>
      {hitIndex < 0 && <ResultBubble text={fallback} />}
    </div>
  );
}
