import { useState, useEffect } from 'react';
import { Download } from 'lucide-react';
import JSZip from 'jszip';

interface AvailableFormat {
  name: string;
  version: string;
  displayName: string;
}

function generateIFID(): string {
  const hex = '0123456789ABCDEF';
  const pattern = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx';
  return pattern.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return hex[v];
  });
}

function resolveBase(): string {
  return import.meta.env.DEV ? '/' : '/TweeRS/';
}

async function scanAvailableFormats(): Promise<AvailableFormat[]> {
  const base = resolveBase();
  const knownFormats = [
    { name: 'SugarCube', version: '2.37.3' },
    { name: 'Harlowe', version: '4-unstable' },
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
          displayName: `${format.name} ${format.version}`,
        });
      }
    } catch {
      // 格式不存在，跳过
    }
  }
  return formats;
}

function generateTweeContent(formatName: string, formatVersion: string): string {
  const ifid = generateIFID();

  if (formatName.toLowerCase() === 'sugarcube') {
    return `:: StoryTitle
我的故事

:: StoryData
{
  "ifid": "${ifid}",
  "format": "${formatName}",
  "format-version": "${formatVersion}"
}

:: Start
你站在一间昏暗的房间里，四周弥漫着灰尘的气味。

房间中央有一张桌子，上面放着一封信和一把钥匙。

[[拿起信|Letter]]
[[拿起钥匙|Key]]

:: Letter
你展开信纸，上面写着：

<blockquote>"钥匙会带你找到答案。"</blockquote>

[[放下信，回到桌前|Start]]

:: Key
你拿起钥匙，发现房间角落有一扇上锁的门。

[[用钥匙开门|Ending]]
[[放下钥匙|Start]]

:: Ending
门缓缓打开，阳光涌入房间。

你走了出去，迎接崭新的世界。

''— 完 —''
`;
  }

  // Harlowe
  return `:: StoryTitle
我的故事

:: StoryData
{
  "ifid": "${ifid}",
  "format": "${formatName}",
  "format-version": "${formatVersion}"
}

:: Start
你站在一间昏暗的房间里，四周弥漫着灰尘的气味。

房间中央有一张桌子，上面放着一封信和一把钥匙。

[[拿起信->Letter]]
[[拿起钥匙->Key]]

:: Letter
你展开信纸，上面写着：

//"钥匙会带你找到答案。"//

[[放下信，回到桌前->Start]]

:: Key
你拿起钥匙，发现房间角落有一扇上锁的门。

[[用钥匙开门->Ending]]
[[放下钥匙->Start]]

:: Ending
门缓缓打开，阳光涌入房间。

你走了出去，迎接崭新的世界。

**— 完 —**
`;
}

export function TemplateDownload() {
  const [formats, setFormats] = useState<AvailableFormat[]>([]);
  const [selected, setSelected] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    scanAvailableFormats().then((f) => {
      setFormats(f);
      if (f.length > 0) {
        setSelected(`${f[0].name}|${f[0].version}`);
      }
    });
  }, []);

  async function handleDownload() {
    if (!selected) return;
    setLoading(true);

    try {
      const [name, version] = selected.split('|');
      const key = `${name.toLowerCase()}-${version}`;
      const base = resolveBase();

      // 加载 format.js
      const formatRes = await fetch(`${base}story-format/${key}/format.js`);
      if (!formatRes.ok) throw new Error('无法加载故事格式文件');
      const formatJs = await formatRes.text();

      // 生成 zip
      const zip = new JSZip();
      zip.file(`demo/.storyformats/${key}/format.js`, formatJs);
      zip.file('demo/src/index.tw', generateTweeContent(name, version));

      const blob = await zip.generateAsync({ type: 'blob' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'demo.zip';
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Download error:', e);
    } finally {
      setLoading(false);
    }
  }

  if (formats.length === 0) return null;

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: '8px',
      marginBlock: '16px',
    }}>
      <select
        value={selected}
        onChange={(e) => setSelected(e.target.value)}
        style={{
          height: '36px',
          padding: '0 12px',
          border: '1px solid #e5e5e5',
          borderRadius: '6px',
          fontSize: '14px',
          cursor: 'pointer',
        }}
      >
        {formats.map((f) => (
          <option key={`${f.name}|${f.version}`} value={`${f.name}|${f.version}`}>
            {f.displayName}
          </option>
        ))}
      </select>
      <button
        onClick={handleDownload}
        disabled={loading}
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '6px',
          height: '36px',
          padding: '0 16px',
          background: loading ? '#999' : '#2c2c2c',
          color: '#fff',
          border: 'none',
          borderRadius: '6px',
          fontSize: '14px',
          cursor: loading ? 'not-allowed' : 'pointer',
        }}
      >
        <Download size={16} />
        {loading ? '生成中…' : '下载模板'}
      </button>
    </div>
  );
}
