if (format.name !== "SugarCube" || format.version !== "2.37.3") 
    return input;

const Colors = {
    "_ele": "green",
    "$backpack['石头']": "blue",
};

Object.values(input).forEach(passage => {
    let content = passage.content;

    Object.entries(Colors).forEach(([variable, color]) => {
        const escapedVariable = variable.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const pattern = new RegExp(
            `(?<!\\$\\.|\\w\\.)\\b(${escapedVariable})(?=\\.|\\b)(?![^<]*>>)`,
            'g'
        );
        
        content = content.replace(pattern, `<span style="color: ${color};">$1</span>`);
    });
    passage.content = content;
});

return input;
