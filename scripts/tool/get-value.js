/**
 * 获取指定值的工具类
 * 用于从 tweers-exceldata 中获取 Excel 生成的数据
 * 
 * 使用示例:
 *   GetValue.byId('abc-1', 'name')      // 通过 id 获取 data-name 属性
 *   GetValue.byId('abc-1', 'content')   // 通过 id 获取 <content> 子标签内容
 *   GetValue.byName('abc', '测试', 'content')  // 通过 name 查找第一个匹配的元素
 */
(function() {
    'use strict';

    // 从元素获取字段值的通用方法
    function getFieldValue(element, field) {
        if (!element) return null;
        const child = element.querySelector(field);
        if (child) return child.textContent;
        if (field === 'name') return element.getAttribute('data-name');
        return element.getAttribute(`data-${field}`);
    }

    // 从 id 提取 saveName (如 "abc-1" -> "abc")
    function extractSaveName(id) {
        const lastDash = id.lastIndexOf('-');
        return lastDash > 0 ? id.substring(0, lastDash) : null;
    }

    const GetValue = {
        /** 通过 id 获取值 */
        byId: function(id, field) {
            const saveName = extractSaveName(id);
            if (!saveName) return null;
            const container = document.querySelector(`tweers-exceldata > ${saveName}`);
            if (!container) return null;
            const element = container.querySelector(`#${id}`);
            return element ? getFieldValue(element, field) : null;
        },

        /** 通过 name 查找第一个匹配的元素并获取值 */
        byName: function(saveName, name, field) {
            const container = document.querySelector(`tweers-exceldata > ${saveName}`);
            if (!container) return null;
            const elements = Array.from(container.querySelectorAll('div[id]'));
            for (const el of elements) {
                const nameValue = getFieldValue(el, 'name');
                if (nameValue === name) {
                    return getFieldValue(el, field);
                }
            }
            return null;
        }
    };

    if (typeof window !== 'undefined') window.GetValue = GetValue;
    if (typeof setup !== 'undefined' && typeof setup.macro !== 'undefined') setup.GetValue = GetValue;
})();
