# TweeRS
> Twee / Twine 文件构建与打包工具，支持脚本注入与 Excel 数据导出  
> [脚本示例](./scripts/Scripts.md)

当前版本: `1.0.1`

## 项目简介
- 将 `.twee` / `.tw` 转为 HTML，支持监听、起始片段指定、媒体 Base64 打包。
- `pack` 模式可同时压缩资源，适合分发发布。
- 内置脚本注入管线，可在读取 Twee 或生成 HTML 后执行自定义 JS。
- Excel 数据转 JS/HTML：支持对象表、参数表与 HTML 表，生成 JS 赋值或嵌入 `<tweers-exceldata>`。

## 安装
- 下载 Release 可执行文件并置于任意目录（建议与 `story-format/`、`scripts/` 同级）。
- 或源码构建：`cargo build --release`，产物位于 `target/release/tweers`。
- Rspress 文档站开发：进入 `docs/` 后 `pnpm install && pnpm run dev`。

## 快速开始
```bash
# 构建 HTML
tweers build story/ -o dist/index.html

# 监听模式
tweers build story/ -w -o dist/index.html

# Base64 打包媒体并指定起始片段
tweers build story/ -b -s Start

# 构建并打包资源
tweers pack story/ -a assets/ -o package.zip
```

## 命令说明
### build
- `tweers build <source_dir> [-o <output>][-s <start>][-b][-w][-t]`
- 主要参数：
  - `-o, --output-path` 输出 HTML，默认 `index.html`
  - `-s, --start-passage` 指定起始片段
  - `-b, --base64` 媒体转 Base64 嵌入
  - `-w, --watch` 监听源文件变更
  - `-t, --is-debug` 输出调试日志

### pack
- `tweers pack <source_dir> -a <assets_dir>... [-o <zip>][-f][-t]`
- 主要参数：
  - `-a, --assets` 需压缩的资源目录，可多次指定
  - `-o, --output-path` 输出压缩包，默认 `package.zip`
  - `-f, --fast-compression` 快速压缩（低质量高速度）
  - `-t, --is-debug` 调试日志
> 可选安装 ffmpeg 以获得更好的音视频压缩体验。

### update
- `tweers update [-f]`：更新到最新发布版（`-f` 强制更新）。

## Excel 模板速览
- 表头行以 `#` 开头；对象表需要 `#save`、`#obj`、`#type` 三行；参数表需要 `#save`、`#var`；HTML 表需要 `#save`、`#html`。
- 保存变量写在 `#save` 后，支持三类模板：
  - `all#Target($content)`：批量生成数组并传入模板中的 `$content`。
  - `single#Target($name,{displayName:$displayName},$tags)`：逐行展开占位符。
  - 直接变量名（如 `window.items`）：生成 `window.items = [...]`。
- HTML 表会生成 `<tweers-exceldata><saveName>...</saveName></tweers-exceldata>`，`id` 固定为 `<saveName>-<行号>`，`name` 列会写入 `data-name`。
- 示例与工具：见 `test/excel/example.xlsx`、`scripts/tool/get-value.js`（提供 `GetValue.byId/byName` 读取 HTML 数据）。

## 脚本注入
- 在 `scripts/` 下放置数据注入脚本（处理 Twee 数据）或 HTML 注入脚本（生成后替换）。
- 示例：
  - `scripts/html/sugarcube/save-slots.js`：修改存档插槽上限。
  - `scripts/data/sugarcube/i18.js`：按语言前缀过滤/重命名片段。
  - `scripts/tool/get-value.js`：运行时读取 `<tweers-exceldata>`。

## 特性状态
- [x] 正则匹配与 JS 注入
- [x] Excel 支持（对象/参数/HTML 表）
- [x] 媒体压缩（图片/音频/视频）
- [ ] Twine 1 / Harlowe 支持