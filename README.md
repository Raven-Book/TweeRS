# TweeRS
> 本项目目前处于实验阶段，功能尚未稳定，不建议使用
>
> 不再支持 `linux_arm64`

当前版本: `0.1.7-1`

## 1. 项目简介

## 2. 常用命令

### 2.1. 命令参数说明

#### 2.1.1 build 命令
将 Twee 格式的故事文件构建为 HTML 输出。

**语法：**

```bash
tweers build <source_dir> -o <output_dir> -t -b -w
```

**参数：**

- `<source_dir>`：输入文件路径
- `-o <output_dir>`：输出文件路径
- `-b` 将资源文件转为base64打包在片段中
- `-w` 监听文件变化
- `-t` 测试模式

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

## 4. Features
- [x] 增加正则匹配模块与JS注入模块
- [ ] 支持 import/export 语法, 以控制 JavaScript/CSS 资源加载顺序
- [ ] 修复文件监听和异步处理中的逻辑问题
- [ ] 重构项目架构以支持 NPM 包管理
- [ ] 兼容 Twine 1 格式文件
- [ ] 支持 Harlowe 故事格式
- [ ] 完善英文文档
- [ ] 集成 NPM 包支持
- [ ] 支持图片/音频/视频等媒体资源压缩

## 5. Link
- Q群: 1044470765