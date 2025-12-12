# Excel 模板与导出

TweeRS 支持将 Excel 转为 JS 变量或 HTML 片段。表头行需以 `#` 开头，从首个 `#` 行到最后一个 `#` 行视为表头，其余为数据行。

## 三类表
| 类型 | 必需表头 | 说明 |
| --- | --- | --- |
| 对象表 | `#save`、`#obj`、`#type` | 生成 JS 对象数组，可按模板写入不同形式 |
| 参数表 | `#save`、`#var` | 生成单个对象 `{ name: value }` |
| HTML 表 | `#save`、`#html` | 生成 `<tweers-exceldata>` 结构供运行时读取 |

### 对象表
- `#save` 后写入保存变量或模板，例如 `all#Item.addAll($content)`。
- `#obj` 后依次填写列名；`#type` 后依次填写类型（`int`、`float`、`string`、`array<string>` 等）。
- 数据行从表头结束的下一行开始，按列填值。

支持的保存模板：
- `all#Target($content)`：将整表转为数组并替换 `$content`，如 `all#Item.addAll($content)`。
- `single#Target($name,{displayName:$displayName},$tags)`：逐行展开占位符生成函数调用。
- 直接变量名（如 `window.items`）：输出 `window.items = [...]`。

### 参数表
- `#save` 后写入变量名，例如 `window.config`。
- `#var` 后列出字段名（必须包含 `name`、`type`、`value`，可选 `comment`）。
- 生成结果类似：
  ```
  window.config = {
      foo: "bar",
      count: 3,
  };
  ```

### HTML 表
- `#save` 后写入保存名，例如 `abc`。
- `#html` 后列出字段名；数据行按列填值。
- 生成结构：
  ```
  <tweers-exceldata hidden>
    <abc>
      <div id="abc-1" data-name="测试">
        <content>今天是xxx</content>
      </div>
    </abc>
  </tweers-exceldata>
  ```
  - `id` 固定为 `<saveName>-<行号>`。
  - `name` 列会写入 `data-name`，其余列生成同名子标签并自动转义内容。

## 运行时读取
- JS 工具：`scripts/tool/get-value.js` 会在全局注入 `GetValue`。
- 常用方法：
  - `GetValue.byId('abc-1', 'content')`
  - `GetValue.byName('abc', '测试', 'content')`

## 示例
- 参考 `test/excel/example.xlsx` 获取完整表头与数据示例。

