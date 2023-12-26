use regex::Regex;

use super::CodeExtractor;

pub struct CodeBlock {
    language: String,
    code: String,
}

pub struct FencedCodeBlockExtractor;

impl CodeExtractor<String> for FencedCodeBlockExtractor {
    type CodeBlock = CodeBlock;

    fn extract_code_blocks(&self, messages: impl Iterator<Item = String>) -> Vec<Self::CodeBlock> {
        let string = messages.collect::<Vec<_>>().join("");

        eprintln!("string: {}", string);

        let re = Regex::new(r"```(?P<language>\w+)\n(?s)(?P<code>.+?)\n```").unwrap();

        let mut code_blocks = vec![];

        for cap in re.captures_iter(&string) {
            let language = cap.name("language").unwrap().as_str().to_string();
            let code = cap.name("code").unwrap().as_str().to_string();

            code_blocks.push(CodeBlock { language, code });
        }

        code_blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_blocks() {
        let messages = vec![
            "```rust\nfn main() {\nprintln!(\"Hello, world!\");\n}\n```".to_string(),
            "```python\nprint(\"Hello, world!\")\n```".to_string(),
        ];

        let extractor = FencedCodeBlockExtractor;

        let code_blocks = extractor.extract_code_blocks(messages.into_iter());

        assert_eq!(code_blocks.len(), 2);
        assert_eq!(code_blocks[0].language, "rust");
        assert_eq!(
            code_blocks[0].code,
            "fn main() {\nprintln!(\"Hello, world!\");\n}"
        );
        assert_eq!(code_blocks[1].language, "python");
        assert_eq!(code_blocks[1].code, "print(\"Hello, world!\")");
    }
}
