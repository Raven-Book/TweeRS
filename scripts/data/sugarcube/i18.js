if (format.name !== "SugarCube" || format.version !== "2.37.3") 
    return input;

const SUPPORTED_LANGUAGES = [
    "zh",
    "en",
    "fr",
    // 自行添加
];

// 当前语言
const language = "zh";
// 分割符
const delimiter = "_";

const passages = {};

Object.values(input).forEach(passage => {
    const prefix = SUPPORTED_LANGUAGES.some(lang => 
        passage.name.startsWith(lang + delimiter)
    );

    if (prefix) {
        const [lang, ...arrs] = passage.name.split(delimiter);
        const name = arrs.join(delimiter);
        
        if (lang === language) {
            passages[name] = {
                ...passage,
                name: name
            };
        }
    } 

    else {
        passages[passage.name] = passage;
    }
});

return passages;