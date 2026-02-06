import { useState, useEffect, useRef } from 'react';
import { Play, Bug, Maximize } from 'lucide-react';
import { TextareaWithScrollbar } from './TextareaWithScrollbar';
import * as tweers from 'tweers-core';

interface Props {
  code: string;
  height?: string;
  hideStoryHeader?: boolean;
}

interface FormatInfo {
  name: string;
  version: string;
}

interface AvailableFormat {
  name: string;
  version: string;
  displayName: string;
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
  return code.replace(/"ifid":\s*"[A-F0-9-]+"/i, `"ifid": "${generateIFID()}"`);
}

// 替换 StoryData 中的故事格式
function replaceStoryFormat(
  code: string,
  name: string,
  version: string,
): string {
  let result = code;
  // 替换 format
  result = result.replace(/"format":\s*"[^"]*"/i, `"format": "${name}"`);
  // 替换 format-version
  result = result.replace(
    /"format-version":\s*"[^"]*"/i,
    `"format-version": "${version}"`,
  );
  return result;
}

// 从 Twee 代码中分离 StoryTitle 和 StoryData 片段
function splitStoryHeader(code: string): { header: string; body: string } {
  const passages = code.split(/^(?=:: )/m);
  const headerParts: string[] = [];
  const bodyParts: string[] = [];

  for (const passage of passages) {
    if (/^:: Story(Title|Data)\b/.test(passage)) {
      headerParts.push(passage);
    } else {
      bodyParts.push(passage);
    }
  }

  return {
    header: headerParts.join('').trim(),
    body: bodyParts.join('').trim(),
  };
}

// 从代码中提取当前格式
function getCurrentFormat(code: string): string {
  const formatMatch = code.match(/"format":\s*"([^"]*)"/i);
  const versionMatch = code.match(/"format-version":\s*"([^"]*)"/i);
  if (formatMatch && versionMatch) {
    return `${formatMatch[1].toLowerCase()}|${versionMatch[1]}`;
  }
  return '';
}

export function TweePlayground({
  code,
  height = '280px',
  hideStoryHeader = false,
}: Props) {
  const [storyHeader] = useState(() => {
    if (!hideStoryHeader) return '';
    return splitStoryHeader(replaceIFID(code.trim())).header;
  });
  const [input, setInput] = useState(() => {
    const full = replaceIFID(code.trim());
    if (!hideStoryHeader) return full;
    return splitStoryHeader(full).body;
  });
  const [html, setHtml] = useState('');
  const [error, setError] = useState('');
  const [isDebug, setIsDebug] = useState(false);
  const [availableFormats, setAvailableFormats] = useState<AvailableFormat[]>(
    [],
  );
  const iframeRef = useRef<HTMLIFrameElement>(null);

  // 扫描可用的故事格式
  async function scanAvailableFormats(): Promise<AvailableFormat[]> {
    const base = import.meta.env.DEV ? '/' : '/TweeRS/';
    const knownFormats = [
      { name: 'harlowe', version: '4-unstable' },
      { name: 'sugarcube', version: '2.37.3' },
    ];

    const formats: AvailableFormat[] = [];
    for (const format of knownFormats) {
      const key = `${format.name.toLowerCase()}-${format.version}`;
      const path = `${base}story-format/${key}/format.js`;

      try {
        const response = await fetch(path, { method: 'HEAD' });
        if (response.ok) {
          formats.push({
            name: format.name,
            version: format.version,
            displayName: `${format.name.charAt(0).toUpperCase() + format.name.slice(1)} ${format.version}`,
          });
        }
      } catch (e) {
        // 格式不存在，跳过
      }
    }
    return formats;
  }

  // 加载故事格式文件
  async function loadFormat(formatInfo: FormatInfo): Promise<string> {
    const key = `${formatInfo.name.toLowerCase()}-${formatInfo.version}`;
    const base = import.meta.env.DEV ? '/' : '/TweeRS/';
    const path = `${base}story-format/${key}/format.js`;

    const response = await fetch(path);
    if (!response.ok) {
      throw new Error(`无法加载格式文件: ${path}`);
    }

    return await response.text();
  }

  // 获取完整的 Twee 代码（拼接隐藏的头部）
  function getFullSource(source: string): string {
    if (!storyHeader) return source;
    return storyHeader + '\n\n' + source;
  }

  // 构建 HTML
  async function build(source: string, debug?: boolean) {
    const useDebug = debug ?? isDebug;
    setError('');
    try {
      const fullSource = getFullSource(source);
      // 解析 Twee 代码，返回包含 format_info（source 为空）的结果
      const parsed = tweers.parse([
        { type: 'text', name: 'story.twee', content: fullSource },
      ]);

      // 从 story_data 中获取格式信息
      const formatInfo: FormatInfo = {
        name: parsed.story_data.format,
        version: parsed.story_data['format-version'],
      };

      // 加载对应的格式文件
      const formatSource = await loadFormat(formatInfo);

      // 填充 format_info
      parsed.format_info.name = formatInfo.name;
      parsed.format_info.version = formatInfo.version;
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

  // 切换故事格式
  function handleFormatChange(formatKey: string) {
    if (!formatKey) return;

    const [name, version] = formatKey.split('|');
    const newInput = replaceStoryFormat(input, name, version);
    setInput(newInput);
    build(newInput).catch((err) => setError(String(err)));
  }

  // 初始化：扫描可用格式
  useEffect(() => {
    scanAvailableFormats().then((formats) => {
      setAvailableFormats(formats);
    });
  }, []);

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
    select: {
      height: '28px',
      padding: '0 8px',
      background: '#f5f5f5',
      border: '1px solid #e5e5e5',
      borderRadius: '4px',
      fontSize: '12px',
      cursor: 'pointer',
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
          {availableFormats.length > 0 && !hideStoryHeader && (
            <select
              value={getCurrentFormat(input)}
              onChange={(e) => handleFormatChange(e.target.value)}
              style={styles.select}
              title="选择故事格式"
            >
              {availableFormats.map((format) => (
                <option
                  key={`${format.name}|${format.version}`}
                  value={`${format.name}|${format.version}`}
                >
                  {format.displayName}
                </option>
              ))}
            </select>
          )}
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
