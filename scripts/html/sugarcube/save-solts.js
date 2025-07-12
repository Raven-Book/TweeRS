if (format.name !== "SugarCube" || format.version !== "2.37.3") 
    return input;


const MAX_INDEX = 9999;

return input.replace(/(MAX_INDEX=)(\d+)/, `$1${MAX_INDEX}`);