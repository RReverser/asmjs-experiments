const NAMED_ESCAPES = {
    '': '::',
    'C': ',',
    'SP': '@',
    'BP': '*',
    'RF': '&',
    'LT': '<',
    'GT': '>',
    'LP': '(',
    'RP': ')'
};

module.exports = function demangle(s, withHash) {
    if (s.startsWith('_')) {
        s = s.slice(1);
    } else {
        return s;
    }

    let match = s.match(/^_?ZN(.*)17h[0-9a-f]{16}E$/);
    if (match) {
        s = match[1];
    } else {
        return s;
    }

    let result = [];

    for (let i = 0; i < s.length;) {
        let length = 0;
        for (; i < s.length; i++) {
            let digit = s.charCodeAt(i) - 48;
            if (digit < 0 || digit > 9) {
                break;
            }
            length = length * 10 + digit;
        }

        let component = s.slice(i, i += length);

        if (component.startsWith('_')) {
            component =
                component.slice(1)
                .replace(/_u([0-9a-f]{2})_/g, (match, code) => String.fromCharCode(parseInt(code, 16)))
                .replace(/_([A-Z]*)_/g, (match, name) => NAMED_ESCAPES[name] || match);
        }

        result.push(component);
    }

    return result.join('::');
};