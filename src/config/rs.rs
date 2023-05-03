use crate::Syntax;
use crate::SyntaxModes;

pub fn init() -> Syntax {
    return Syntax {
        keywords: vec![
            "as".to_string(),
            "break".to_string(),
            "const".to_string(),
            "continue".to_string(),
            "crate".to_string(),
            "else".to_string(),
            "enum".to_string(),
            "extern".to_string(),
            "false".to_string(),
            "fn".to_string(),
            "for".to_string(),
            "if".to_string(),
            "impl".to_string(),
            "in".to_string(),
            "let".to_string(),
            "loop".to_string(),
            "match".to_string(),
            "mod".to_string(),
            "move".to_string(),
            "mut".to_string(),
            "pub".to_string(),
            "ref".to_string(),
            "return".to_string(),
            "self".to_string(),
            "Self".to_string(),
            "static".to_string(),
            "struct".to_string(),
            "super".to_string(),
            "trait".to_string(),
            "true".to_string(),
            "type".to_string(),
            "unsafe".to_string(),
            "use".to_string(),
            "where".to_string(),
            "while".to_string(),
        ],
        symbols: vec![
            ".".to_string(),
            ",".to_string(),
            " ".to_string(),
            "(".to_string(),
            ")".to_string(),
            "{".to_string(),
            "}".to_string(),
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/".to_string(),
            "=".to_string(),
            ":".to_string(),
            ";".to_string(),
            "@".to_string(),
            "<".to_string(),
            ">".to_string(),
            "&".to_string(),
            "\"".to_string(),
        ],
        types: vec![
            "u8".to_string(),
            "u16".to_string(),
            "u32".to_string(),
            "u64".to_string(),
            "u128".to_string(),
            "usize".to_string(),
            "i8".to_string(),
            "i16".to_string(),
            "i32".to_string(),
            "i64".to_string(),
            "i128".to_string(),
            "isize".to_string(),
            "f32".to_string(),
            "f64".to_string(),
            "bool".to_string(),
            "char".to_string(),
            "str".to_string(),
            "String".to_string(),
        ],
        operators: vec![
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/".to_string(),
            "=".to_string(),
            ">".to_string(),
            "<".to_string(),
            "%".to_string(),
        ],
        string: "\"".to_string(),
        comment: "//".to_string(),
        mode: SyntaxModes::Normal,
    }
}

