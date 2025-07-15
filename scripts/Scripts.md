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