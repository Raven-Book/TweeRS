import { useState } from 'react';
import { ScrollArea } from '@/components/ScrollArea';

const BORDER = '#dbe3ec';
const HEADER_BG = '#d9e8ff';
const HEADER_FG = '#153b73';
const COL_HEADER_BG = '#eef4fb';
const ROW_NUM_BG = '#f8fbff';
const SHEET_BG = '#f7fbff';
const OUTPUT_BG = '#f8fafc';
const OUTPUT_FG = '#0f172a';

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    marginBlock: '16px',
    border: `1px solid ${BORDER}`,
    borderRadius: '10px',
    overflow: 'hidden' as const,
    background: '#ffffff',
    boxShadow: '0 16px 32px rgba(15, 23, 42, 0.08)',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '12px',
    padding: '12px 16px',
    borderBottom: `1px solid ${BORDER}`,
    background:
      'linear-gradient(180deg, rgba(255,255,255,0.98) 0%, rgba(245,248,252,0.98) 100%)',
  },
  headerCopy: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '2px',
  },
  title: {
    fontSize: '15px',
    fontWeight: 700,
    color: '#12263f',
  },
  subtitle: {
    fontSize: '12px',
    lineHeight: 1.5,
    color: '#70839d',
  },
  headerActions: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    flexShrink: 0,
  },
  controls: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '6px',
    padding: '4px',
    borderRadius: '8px',
    background: '#edf3f9',
    border: `1px solid ${BORDER}`,
    flexShrink: 0,
  },
  controlBtn: {
    padding: '7px 12px',
    border: 'none',
    borderRadius: '6px',
    background: 'transparent',
    color: '#56708f',
    fontSize: '12px',
    fontWeight: 700,
    cursor: 'pointer',
    lineHeight: 1,
  },
  controlBtnActive: {
    background: '#ffffff',
    color: '#173d74',
    boxShadow: '0 1px 2px rgba(15, 23, 42, 0.08)',
  },
  sheetViewport: {
    background: SHEET_BG,
  },
  sheetTrack: {
    right: '4px',
  },
  sheetThumb: {
    background: 'rgba(37, 99, 235, 0.38)',
  },
  tableWrap: {
    minWidth: 'max-content',
  },
  table: {
    borderCollapse: 'separate' as const,
    borderSpacing: 0,
    minWidth: '100%',
    fontSize: '13px',
    fontFamily:
      '"SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace',
    background: '#ffffff',
  },
  cornerCell: {
    position: 'sticky' as const,
    top: 0,
    left: 0,
    zIndex: 4,
    width: '52px',
    minWidth: '52px',
    background: COL_HEADER_BG,
    borderRight: `1px solid ${BORDER}`,
    borderBottom: `1px solid ${BORDER}`,
  },
  colHeader: {
    position: 'sticky' as const,
    top: 0,
    zIndex: 3,
    minWidth: '104px',
    padding: '10px 14px',
    textAlign: 'center' as const,
    fontWeight: 700,
    color: '#53657a',
    background: COL_HEADER_BG,
    borderRight: `1px solid ${BORDER}`,
    borderBottom: `1px solid ${BORDER}`,
  },
  rowNum: {
    position: 'sticky' as const,
    left: 0,
    zIndex: 2,
    width: '52px',
    minWidth: '52px',
    padding: '10px 12px',
    textAlign: 'center' as const,
    fontWeight: 700,
    color: '#8a9aae',
    background: ROW_NUM_BG,
    borderRight: `1px solid ${BORDER}`,
    borderBottom: `1px solid ${BORDER}`,
  },
  headerCell: {
    padding: '10px 14px',
    background: HEADER_BG,
    color: HEADER_FG,
    fontWeight: 700,
    whiteSpace: 'nowrap' as const,
    borderRight: '1px solid #bfd5f7',
    borderBottom: '1px solid #bfd5f7',
  },
  dataCell: {
    padding: '10px 14px',
    color: '#1f2937',
    whiteSpace: 'nowrap' as const,
    background: '#ffffff',
    borderRight: `1px solid ${BORDER}`,
    borderBottom: `1px solid ${BORDER}`,
  },
  emptyCellA: {
    padding: '10px 14px',
    background: '#f8fafc',
    borderRight: `1px solid ${BORDER}`,
    borderBottom: `1px solid ${BORDER}`,
  },
  outputViewport: {
    background: OUTPUT_BG,
  },
  outputTrack: {
    right: '4px',
  },
  outputThumb: {
    background: 'rgba(100, 116, 139, 0.42)',
  },
  outputText: {
    margin: 0,
    padding: '14px 18px 18px',
    fontSize: '13px',
    lineHeight: 1.75,
    fontFamily:
      '"SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace',
    color: OUTPUT_FG,
    whiteSpace: 'pre-wrap' as const,
    overflowWrap: 'anywhere' as const,
    wordBreak: 'break-word' as const,
  },
};

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function getColumnLetter(index: number): string {
  return String.fromCharCode(65 + index);
}

interface ExcelVisualizerProps {
  headers: string[][];
  data: string[][];
  output: string;
  outputType?: 'js' | 'html';
}

export function ExcelVisualizer({
  headers,
  data,
  output,
  outputType = 'js',
}: ExcelVisualizerProps) {
  const [view, setView] = useState<'sheet' | 'output'>('sheet');
  const allRows = [...headers, ...data];
  const maxCols = allRows.reduce((max, row) => Math.max(max, row.length), 0);
  const outputLines = output.replace(/\n$/, '').split('\n');
  const panelHeight = `${clamp(
    Math.max((allRows.length + 1) * 46, outputLines.length * 24 + 36),
    220,
    340,
  )}px`;

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <div style={styles.headerCopy}>
          <div style={styles.title}>
            {view === 'sheet'
              ? 'Excel 表格'
              : outputType === 'html'
                ? '生成的 HTML'
                : '生成的代码'}
          </div>
          <div style={styles.subtitle}>
            {view === 'sheet'
              ? '支持上下滚动，也可以直接拖拽表格内容进行左右平移。'
              : '自动换行展示，避免横向滚动影响阅读。'}
          </div>
        </div>
        <div style={styles.headerActions}>
          <div style={styles.controls}>
            <button
              type="button"
              onClick={() => setView('sheet')}
              style={{
                ...styles.controlBtn,
                ...(view === 'sheet' ? styles.controlBtnActive : {}),
              }}
            >
              表格
            </button>
            <button
              type="button"
              onClick={() => setView('output')}
              style={{
                ...styles.controlBtn,
                ...(view === 'output' ? styles.controlBtnActive : {}),
              }}
            >
              预览
            </button>
          </div>
        </div>
      </div>

      {view === 'sheet' ? (
        <div>
          <ScrollArea
            height={panelHeight}
            viewportStyle={styles.sheetViewport}
            trackStyle={styles.sheetTrack}
            thumbStyle={styles.sheetThumb}
            dragToScroll
          >
            <div style={styles.tableWrap}>
              <table style={styles.table}>
                <thead>
                  <tr>
                    <th style={styles.cornerCell}></th>
                    {Array.from({ length: maxCols }, (_, i) => (
                      <th key={i} style={styles.colHeader}>
                        {getColumnLetter(i)}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {allRows.map((row, ri) => {
                    const isHeader = ri < headers.length;
                    return (
                      <tr key={ri}>
                        <td style={styles.rowNum}>{ri + 1}</td>
                        {Array.from({ length: maxCols }, (_, ci) => {
                          const val = row[ci] ?? '';
                          if (isHeader) {
                            return (
                              <td key={ci} style={styles.headerCell}>
                                {val}
                              </td>
                            );
                          }
                          if (ci === 0 && val === '') {
                            return <td key={ci} style={styles.emptyCellA}></td>;
                          }
                          return (
                            <td key={ci} style={styles.dataCell}>
                              {val}
                            </td>
                          );
                        })}
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </ScrollArea>
        </div>
      ) : (
        <div>
          <ScrollArea
            height={panelHeight}
            viewportStyle={styles.outputViewport}
            trackStyle={styles.outputTrack}
            thumbStyle={styles.outputThumb}
          >
            <pre style={styles.outputText}>
              <code>{output}</code>
            </pre>
          </ScrollArea>
        </div>
      )}
    </div>
  );
}
