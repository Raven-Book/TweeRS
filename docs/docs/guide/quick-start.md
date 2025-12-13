# 快速开始

## 安装
- 从发布页下载可执行文件，放置在 `tweers` 同级目录包含 `story-format/`、`scripts/`。
- 或源码构建：`cargo build --release`，产物位于 `target/release/tweers`。
- 文档站开发：进入 `docs/` 后执行 `pnpm install && pnpm run dev`。

## 基础命令
```bash
# 构建 HTML
tweers build story/ -o dist/index.html

# 监听模式
tweers build story/ -w -o dist/index.html

# Base64 媒体打包并指定起始片段
tweers build story/ -b -s Start

# 构建并压缩资源
tweers pack story/ -a assets/ -o package.zip
```

### build 参数
- `-o, --output-path` 输出 HTML，默认 `index.html`
- `-s, --start-passage` 指定故事起始片段
- `-b, --base64` 将媒体转 Base64 嵌入
- `-w, --watch` 监听源文件自动重建
- `-t, --is-debug` 输出调试日志

### pack 参数
- `-a, --assets` 需要压缩的资源目录，可多次指定
- `-o, --output-path` 输出压缩包，默认 `package.zip`
- `-f, --fast-compression` 快速压缩（低质量高速度）
- `-t, --is-debug` 调试日志  
> 建议安装 ffmpeg 获得更好的音视频压缩效果。

### update
- `tweers update [-f]`：更新到最新发布版（`-f` 强制更新）。

## 推荐目录结构
```
📂
├── tweers[.exe]
├── story-format/
└── scripts/
    ├── data/    # 处理 Twee 数据
    └── html/    # 处理生成后的 HTML
```

