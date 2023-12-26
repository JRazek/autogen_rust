use regex::Regex;

use super::CodeExtractor;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    Python,
}

impl TryInto<Language> for &str {
    type Error = ();

    fn try_into(self) -> Result<Language, Self::Error> {
        match self {
            "rust" => Ok(Language::Rust),
            "python" => Ok(Language::Python),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub language: Language,
    pub code: String,
}

pub struct FencedCodeBlockExtractor;

impl CodeExtractor<String> for FencedCodeBlockExtractor {
    type CodeBlock = CodeBlock;

    //make it accept stream and a sink. Then it will match code blocks as an async element and send them to the sink
    fn extract_code_blocks(&self, string: String) -> Vec<Self::CodeBlock> {
        let re = Regex::new(r"```(?P<language>\w+)\n(?s)(?P<code>.+?)\n```").unwrap();

        let mut code_blocks = vec![];

        for cap in re.captures_iter(&string) {
            let language = cap.name("language").unwrap().as_str().try_into();

            if let Ok(language) = language {
                eprintln!("language: {:?}", language);
                let code = cap.name("code").unwrap().as_str().to_string();

                code_blocks.push(CodeBlock { language, code });
            }
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

        let code_blocks = messages
            .iter()
            .map(|message| extractor.extract_code_blocks(message.to_string()))
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(code_blocks.len(), 2);
        assert_eq!(code_blocks[0].language, Language::Rust);
        assert_eq!(
            code_blocks[0].code,
            "fn main() {\nprintln!(\"Hello, world!\");\n}"
        );
        assert_eq!(code_blocks[1].language, Language::Python);
        assert_eq!(code_blocks[1].code, "print(\"Hello, world!\")");
    }
}
