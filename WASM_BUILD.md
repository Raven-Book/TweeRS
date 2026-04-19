# WASM Build Guide

## 快速开始

构建后的 WASM 文件会输出到 `target/wasm/` 目录。
当前额外提供 `parse_html(html)` 和 `html_to_twee(html)` API，用于将 Twine 导出 HTML 解析或转换回 Twee。

### Linux/macOS

```bash
# 默认构建（web target, release mode）
./build-wasm.sh

# 开发模式构建
./build-wasm.sh --dev

# 指定目标平台
./build-wasm.sh --target nodejs
./build-wasm.sh --target bundler

# 自定义输出目录
./build-wasm.sh --out-dir wasm-output

# 查看帮助
./build-wasm.sh --help
```

### Windows

```cmd
REM 默认构建（web target, release mode）
build-wasm.bat

REM 开发模式构建
build-wasm.bat --dev

REM 指定目标平台
build-wasm.bat --target nodejs
build-wasm.bat --target bundler

REM 自定义输出目录
build-wasm.bat --out-dir wasm-output

REM 查看帮助
build-wasm.bat --help
```
