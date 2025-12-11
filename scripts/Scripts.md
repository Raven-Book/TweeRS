# Scripts
> 通过 Pull Request 提交你的脚本

## SugarCube 2.37.3

### [save-slots.js](./html/sugarcube/save-slots.js) 

修改最大存档插槽上限

### [i18.js](./data/sugarcube/i18.js)

删除其他语言的片段, 并将指定语言的片段名前缀删除, 如 `zh_片段1` -> `片段1`。

```
:: start
[[片段1]]

:: zh_片段1
片段1

:: en_片段1
passage1
```

### [var_color.js](./data/sugarcube/var_color.js)

通过正则替换指定变量的颜色，修改 `Color` 即可.

```
const Colors = {
    "_ele": "green",
    "$backpack['石头']": "blue",
};
```

## 工具类

### [get-value.js](./tool/get-value.js)

用于从 `tweers-exceldata` 中获取指定值的工具类。

**引入方式：**

将 `get-value.js` 添加到你的项目中，它会在全局作用域创建 `GetValue` 对象。

**主要方法：**

- `GetValue.byId(id, field)` - 通过 id 获取值（id 格式：`saveName-index`，如 `abc-1`）
- `GetValue.byName(saveName, name, field)` - 通过 name 查找第一个匹配的元素并获取值

**使用示例：**

假设你的 Excel 生成了以下 HTML：

```html
<tweers-exceldata>
    <abc>
        <div id="abc-1" data-name="测试">
            <content>今天是xxx</content>
            <function>function test() { return true; }</function>
        </div>
        <div id="abc-2" data-name="测试2">
            <content>明天是yyy</content>
        </div>
    </abc>
</tweers-exceldata>
```

**在 JavaScript 中使用：**

```javascript
// 1. 通过 id 获取 data-name 属性
const name = GetValue.byId('abc-1', 'name');
// 返回: "测试"

// 2. 通过 id 获取 content 子标签内容
const content = GetValue.byId('abc-1', 'content');
// 返回: "今天是xxx"

// 3. 通过 id 获取 function 子标签内容, 返回的是字符串
const func = GetValue.byId('abc-1', 'function');
// 返回: "function test() { return true; }"

// 4. 通过 name 查找第一个匹配的元素并获取 content
const content2 = GetValue.byName('abc', '测试', 'content');
// 返回: "今天是xxx"
```