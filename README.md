# TweeRS
> 本项目目前处于实验阶段，功能尚未稳定，不建议使用

当前版本: `0.1.1`

## 项目简介

## 命令

### 命令参数说明

#### build 命令
将 Twee 格式的故事文件构建为 HTML 输出。

**语法：**

```bash
tweers build <source_dir> -o <output_dir>
```

**参数：**

- `<source_dir>`：输入文件路径
- `-o <output_dir>`：输出文件路径
- `-w` 监听目录，当文件发生变化时，自动重新构建。


## Features
- [ ] 增加正则匹配模块与JS注入模板
- [ ] 支持 import/export 语法, 以控制 JavaScript/CSS 资源加载顺序
- [ ] 修复文件监听和异步处理中的逻辑问题
- [ ] 重构项目架构以支持 NPM 包管理
- [ ] 兼容 Twine 1 格式文件
- [ ] 支持 Harlowe 故事格式
- [ ] 完善英文文档
- [ ] 集成 NPM 包支持
- [ ] 支持图片/音频/视频等媒体资源压缩