# 脚本注入

TweeRS 在两个阶段支持自定义脚本：
- **数据阶段**：解析 Twee 后、生成 HTML 前，可修改片段数据。
- **HTML 阶段**：生成 HTML 后，可直接替换/追加内容。

## 目录约定
```
scripts/
├── data/    # 数据阶段脚本
└── html/    # HTML 阶段脚本
```
与可执行文件放在同级目录（通常与 `story-format/` 同级）。

## 数据阶段示例
```js
if (format.name === "SugarCube" && format.version === "2.37.3") {
  for (let passageName in input) {
    const passage = input[passageName];
    if (passageName.includes("事件")) {
      passage.tags ??= "";
      if (!passage.tags.includes("事件")) {
        const tags = passage.tags.trim().split(/\s+/).filter(Boolean);
        tags.push("event");
        passage.tags = tags.join(" ");
      }
    }
  }
}
return input;
```

## HTML 阶段示例
```js
const customStyles = `<style>.macro-button:hover{transform:translateY(-2px);}</style>`;
const idx = input.indexOf('</head>');
if (idx !== -1) {
  input = input.slice(0, idx) + customStyles + '\n' + input.slice(idx);
}
return input;
```

## 已内置/示例脚本
- `scripts/html/sugarcube/save-slots.js`：调整 SugarCube 存档槽位。
- `scripts/data/sugarcube/i18.js`：按语言前缀过滤并去除前缀。
- `scripts/data/sugarcube/var_color.js`：正则替换指定变量的颜色。
- `scripts/tool/get-value.js`：提供 `GetValue.byId/byName` 读取 `<tweers-exceldata>`。

> 欢迎提交通用脚本到 `scripts/` 目录。

