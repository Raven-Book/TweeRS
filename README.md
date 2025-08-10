# TweeRS
> [Scrips](./scripts/Scripts.md) 内有可供使用的注入脚本

当前版本: `0.1.13`

## 1. 项目简介

## 2. 常用命令

### 2.1. 命令参数说明

#### 2.1.1 build 命令
将 Twee 文件构建为 HTML 并输出

**语法：**

```bash
tweers build <source_dir> [OPTIONS]
```

**参数：**

- `<source_dir>`：输入文件路径（必需）
- `-o, --output-path <output_dir>`：输出文件路径（默认：`index.html`）
- `-s, --start-passage <passage_name>`：指定故事的起始片段
- `-b, --base64`：将资源文件转为 base64 打包在片段中
- `-w, --watch`：启用文件监听模式，自动重新构建
- `-t, --is-debug`：启用调试模式，输出详细日志信息

**示例：**

```bash
# 基本构建
tweers build story/

# 指定输出路径
tweers build story/ -o dist/index.html

# 启用 base64 模式打包媒体文件
tweers build story/ -o dist/index.html -b

# 启用监听模式
tweers build story/ -w

# 启用调试模式
tweers build story/ -t

# 指定起始片段
tweers build story/ -s Start

# 组合使用多个选项
tweers build story/ -o dist/index.html -b -w -t -s "Prologue A"
```

#### 2.1.2 pack 命令
> [可选] 下载 [ffmpeg](https://ffmpeg.org/) 支持音视频文件压缩

构建 HTML 并压缩资源打包文件
**语法：**

```bash
tweers pack <source_dir> [OPTIONS]
```

**参数：**

- `<source_dir>`：输入文件路径（必需）
- `-a, --assets <assets_dir>`：需要压缩的资源目录路径（可指定多个）
- `-o, --output-path <output_path>`：输出压缩包路径（默认：`package.zip`，自动使用故事标题命名）
- `-f, --fast-compression`：启用快速压缩模式
- `-t, --is-debug`：启用调试模式，输出详细日志信息

**示例：**

```bash
# 基本打包（自动命名为故事标题.zip）
tweers pack story/ -a assets/

# 指定多个资源目录
tweers pack story/ -a images/ -a audio/ -a videos/

# 指定输出文件名
tweers pack story/ -a assets/ -o my-story.zip

# 启用快速压缩
tweers pack story/ -a assets/ -f

# 启用调试模式
tweers pack story/ -a assets/ -t

# 组合使用多个选项
tweers pack story/ -a assets/ -o my-story.zip -f -t
```

## 3. twee 注入
> 欢迎投稿 `twee` 通用注入脚本

注入分为两种。读取完 twee 文件后执行注入脚本，或是生成完 html 后替换内容。

- 情况1
    ```js
    if (format.name === "SugarCube" && format.version === "2.37.3") {
        for (let passageName in input) {
            let passage = input[passageName];
            
            if (passageName.includes("事件")) {
                if (!passage.tags) {
                    passage.tags = "";
                }
                
                if (!passage.tags.includes("事件")) {
                    const tagsArray = passage.tags.trim().split(/\s+/);
                    tagsArray.push("event");
                    passage.tags = tagsArray.join(' '); 
                    console.log(`Added "事件" tag to passage: ${passageName}`);
                }        
                console.log(JSON.stringify(passage));
            }        
        }
    }
    
    return input;
    ```
- 情况2
    ```js
    const customStyles = `
    <style>
    /* Custom styles for enhanced UI */
    .macro-button:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0,0,0,0.2);
    }
    </style>`;
    
    const headCloseIndex = input.indexOf('</head>');
    if (headCloseIndex !== -1) {
        input = input.slice(0, headCloseIndex) + customStyles + '\n' + input.slice(headCloseIndex);
        console.log('Added custom styles to head section');
    } else {
        console.log('Warning: </head> tag not found, could not add styles');
    }
    
    return input;
    ```
---
与可执行文件同级的的 `scripts` 文件夹下可以放脚本:
```
📂
├── tweers[.exe]        - 可执行文件
├── story-format/       - 故事格式目录
└── scripts/            - 脚本目录
    ├── data/
    │   ├── 01-toc.js       - 自动生成目录
    │   ├── 02-navigation.js - 导航处理
    │   └── 10-i18n.js      - 国际化脚本
    └── html/
        └── 01-theme.js     - 主题样式注入
```

## 5. Features
- [x] 增加正则匹配模块与JS注入模块
- [ ] 支持 import/export 语法, 以控制 JavaScript/CSS 资源加载顺序
- [ ] 修复文件监听和异步处理中的逻辑问题
- [ ] 重构项目架构以支持 NPM 包管理
- [ ] 兼容 Twine 1 格式文件
- [ ] 支持 Harlowe 故事格式
- [ ] 完善英文文档
- [ ] 集成 NPM 包支持
- [ ] javascript 压缩混淆
- [x] 支持图片/音频/视频等媒体资源压缩
- [x] 支持 Excel 文件

## 7. Link
- Q群: 1044470765