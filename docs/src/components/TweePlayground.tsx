import { useState, useEffect, useRef } from 'react';
import { Play, Bug, Maximize } from 'lucide-react';
import { TextareaWithScrollbar } from './TextareaWithScrollbar';
import * as tweers from 'tweers-core';

interface Props {
  code: string;
  height?: string;
}

interface FormatInfo {
  name: string;
  version: string;
}

// 生成 v4 UUID（大写字母）
function generateIFID(): string {
  const hex = '0123456789ABCDEF';
  const pattern = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx';
  return pattern.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return hex[v];
  });
}

// 替换代码中的 IFID
function replaceIFID(code: string): string {
  return code.replace(
    /"ifid":\s*"[A-F0-9-]+"/i,
    `"ifid": "${generateIFID()}"`
  );
}

export function TweePlayground({ code, height = '280px' }: Props) {
  const [input, setInput] = useState(() => replaceIFID(code.trim()));
  const [html, setHtml] = useState('');
  const [error, setError] = useState('');
  const [isDebug, setIsDebug] = useState(false);
  const formatCache = useRef<Map<string, string>>(new Map());
  const iframeRef = useRef<HTMLIFrameElement>(null);

  // 加载故事格式文件
  async function loadFormat(formatInfo: FormatInfo): Promise<string> {
    const key = `${formatInfo.name.toLowerCase()}-${formatInfo.version}`;

    if (formatCache.current.has(key)) {
      return formatCache.current.get(key)!;
    }

    const base = import.meta.env.BASE_URL || '/';
    const path = `${base}story-format/${key}/format.js`;

    const response = await fetch(path);
    if (!response.ok) {
      throw new Error(`无法加载格式文件: ${path}`);
    }

    const text = await response.text();
    formatCache.current.set(key, text);
    return text;
  }

  // 构建 HTML
  async function build(source: string, debug?: boolean) {
    const useDebug = debug ?? isDebug;
    setError('');
    try {
      // 解析 Twee 代码，返回包含 format_info（source 为空）的结果
      const parsed = tweers.parse([
        { type: 'text', name: 'story.twee', content: source },
      ]);

      // 从解析结果中获取格式信息
      const formatInfo: FormatInfo = {
        name: parsed.format_info.name,
        version: parsed.format_info.version,
      };

      // 加载对应的格式文件
      const formatSource = await loadFormat(formatInfo);

      // 填充 format_info.source 和 is_debug
      parsed.format_info.source = formatSource;
      parsed.is_debug = useDebug;

      console.log('Building with format:', formatInfo.name, formatInfo.version);

      // 直接传入 parsed 对象构建 HTML
      const output = tweers.build_from_parsed(parsed);
      setHtml(output.html);
    } catch (e: any) {
      console.error('Build error:', e);
      setError(String(e));
    }
  }

  // 初始化：构建初始代码
  useEffect(() => {
    build(input).catch((err) => setError(String(err)));
  }, []);

  const styles = {
    container: {
      display: 'flex',
      gap: '8px',
      marginBlock: '16px',
    } as const,
    panel: {
      flex: 1,
      position: 'relative' as const,
    },
    textarea: {
      width: '100%',
      height,
      fontFamily: 'monospace',
      fontSize: '13px',
      padding: '12px',
      paddingTop: '40px',
      paddingRight: '16px',
      border: '1px solid #e5e5e5',
      borderRadius: '4px',
      resize: 'none' as const,
      scrollbarWidth: 'none' as const,
    },
    toolbar: {
      position: 'absolute' as const,
      top: '8px',
      right: '8px',
      display: 'flex',
      gap: '4px',
    },
    iconBtn: {
      width: '28px',
      height: '28px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      background: '#f5f5f5',
      border: '1px solid #e5e5e5',
      borderRadius: '4px',
      cursor: 'pointer',
      fontSize: '14px',
    } as const,
    iconBtnActive: {
      background: '#2c2c2c',
      color: '#fff',
      borderColor: '#2c2c2c',
    } as const,
  };

  return (
    <div style={styles.container}>
      <div style={styles.panel}>
        <TextareaWithScrollbar
          value={input}
          onChange={setInput}
          style={styles.textarea}
        />
        <div style={styles.toolbar}>
          <button
            onClick={() => {
              const newDebug = !isDebug;
              setIsDebug(newDebug);
              build(input, newDebug);
            }}
            style={{
              ...styles.iconBtn,
              ...(isDebug ? styles.iconBtnActive : {}),
            }}
            title="Debug 模式"
          >
            <Bug size={16} />
          </button>
          <button
            onClick={() => build(input)}
            style={styles.iconBtn}
            title="运行"
          >
            <Play size={16} />
          </button>
        </div>
      </div>
      <div style={styles.panel}>
        <div style={styles.toolbar}>
          <button
            onClick={() => {
              if (iframeRef.current) {
                iframeRef.current.requestFullscreen();
              }
            }}
            style={styles.iconBtn}
            title="全屏"
          >
            <Maximize size={16} />
          </button>
        </div>
        {error ? (
          <div style={{ color: '#b91c1c', padding: '12px' }}>{error}</div>
        ) : (
          <iframe
            ref={iframeRef}
            srcDoc={html}
            style={{
              width: '100%',
              height,
              border: '1px solid #e5e5e5',
              borderRadius: '4px',
            }}
          />
        )}
      </div>
    </div>
  );
}
