const NUMBER_SUFFIX = '([ui](\d*))?';

const KEYWORDS = [
    "fn",
    "pipeline",
    "entity",
    "inst",
    "mod",
    "match",
    "decl",
    "reg",
    "let",
    "set",
    "stage",
    "struct",
    "enum",
    "port",
    "if",
    "else",
];
const LITERALS = [
    "true",
    "false",
    "Some",
    "None",
];
const BUILTINS = [
    "Option",
    "Some",
    "None",
    "trunc",
    "sext",
    "zext",
    "concat",
    "bit_to_bool",
];
const TYPES = [
    "uint",
    "int",
    "bool",
    "clock",
];

hljs.registerLanguage("spade", (hljs) => ({
    name: "spade",
    keywords: {
        keyword: KEYWORDS.join(" "),
        built_in: "clock uint Option",
        type: TYPES.join(" "),
        literal: LITERALS.join(" "),
    },
    contains: [
        hljs.QUOTE_STRING_MODE,
        {
            className: 'number',
            variants: [
                { begin: '\\b0b([01_]+)' + NUMBER_SUFFIX },
                { begin: '\\b0o([0-7_]+)' + NUMBER_SUFFIX },
                { begin: '\\b0x([A-Fa-f0-9_]+)' + NUMBER_SUFFIX },
                {
                    begin: '\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)'
                        + NUMBER_SUFFIX
                }
            ],
            relevance: 0
        },
        {
            begin: hljs.IDENT_RE + '::',
            keywords: {
                keyword: "Self",
                built_in: BUILTINS.join(" "),
                type: TYPES.join(" ")
            }
        },
        {
            className: "punctuation",
            begin: '->'
        },
        hljs.COMMENT("//.*$"),
    ],
}));

hljs.registerLanguage("error", (hljs) => ({
    name: "error",
    contains: [
        {
            scope: 'error',
            className: "red",
            begin: '^error'
        },
        {
            scope: 'diagnostic_line',
            begin: /\s*\d*\s*│/,
            end: '$',
            className: "blue",
            contains: [
                {
                    scope: 'red',
                    className: "red",
                    begin: /\^.+/,
                    end: /$/,
                },
                {
                    scope: 'green',
                    className: "green",
                    begin: /\s+- .+/,
                    end: /$/,
                },
                {
                    scope: 'suggestion',
                    className: "suggestion",
                    begin: /~(^|\s .)?/,
                    end: /$/,
                },
                {
                    scope: 'red',
                    className: 'red',
                    begin: '[│╰─╭]', end: ''
                },
                {
                    scope: 'unset',
                    className: "unset",
                    begin: /./,
                    end: ''
                },
            ]
        },
        {
            scope: 'blue',
            className: "blue",
            begin: /^\s*┌─.*/,
            end: /$/
        }
    ]
    ,
}));



hljs.initHighlightingOnLoad();
