use crate::{LexerToken, TokenType};

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum LexerAnnotationProcessorInstruction {
    NoOp,
    Drop,
    UntilToken,
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct LexerAnnotationProcessorInfo {
    instruction: LexerAnnotationProcessorInstruction,
    token_type: TokenType,
}

impl LexerAnnotationProcessorInfo {
    pub fn new(instruction: LexerAnnotationProcessorInstruction) -> Self {
        LexerAnnotationProcessorInfo {
            instruction,
            token_type: TokenType::Unknown,
        }
    }

    pub fn drop() -> Self {
        LexerAnnotationProcessorInfo {
            instruction: LexerAnnotationProcessorInstruction::Drop,
            token_type: TokenType::Unknown,
        }
    }

    pub fn noop() -> Self {
        LexerAnnotationProcessorInfo {
            instruction: LexerAnnotationProcessorInstruction::NoOp,
            token_type: TokenType::Unknown,
        }
    }

    pub fn until_token_type(token_type: TokenType) -> Self {
        LexerAnnotationProcessorInfo {
            instruction: LexerAnnotationProcessorInstruction::UntilToken,
            token_type,
        }
    }

    pub fn get_instruction(&self) -> LexerAnnotationProcessorInstruction {
        self.instruction
    }

    pub fn get_token_type(&self) -> TokenType {
        self.token_type
    }
}

pub trait LexerAnnotationProcessor {
    type Error: ToString;

    fn yield_annotation(&mut self, annotation: &String, line: usize, column: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error>;
    fn yield_line_annotation(&mut self, annotation: &String, line: usize, column: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error>;
    fn yield_token(&mut self, token: LexerToken) -> Result<(), Self::Error>;
}

pub struct NoOpProcessor {}

impl NoOpProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl LexerAnnotationProcessor for NoOpProcessor {
    type Error = String;

    fn yield_annotation(&mut self, _: &String, _: usize, _: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
        Ok(LexerAnnotationProcessorInfo::noop())
    }

    fn yield_line_annotation(&mut self, _: &String, _: usize, _: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
        Ok(LexerAnnotationProcessorInfo::noop())
    }

    fn yield_token(&mut self, _: LexerToken) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct DropAnnotationsProcessor {
    dropped_annotations: Vec<LexerToken>,
}

impl DropAnnotationsProcessor {
    pub fn new() -> Self {
        DropAnnotationsProcessor { dropped_annotations: vec![] }
    }

    pub fn get_dropped_annotations(&self) -> &Vec<LexerToken> {
        &self.dropped_annotations
    }
}

impl LexerAnnotationProcessor for DropAnnotationsProcessor {
    type Error = String;

    fn yield_annotation(&mut self, annotation: &String, line: usize, column: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
        self.dropped_annotations
            .push(LexerToken::new(annotation.clone(), TokenType::Annotation, line, column));

        Ok(LexerAnnotationProcessorInfo::new(LexerAnnotationProcessorInstruction::Drop))
    }

    fn yield_line_annotation(&mut self, annotation: &String, line: usize, column: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
        self.dropped_annotations
            .push(LexerToken::new(annotation.clone(), TokenType::LineAnnotation, line, column));

        Ok(LexerAnnotationProcessorInfo::new(LexerAnnotationProcessorInstruction::Drop))
    }

    fn yield_token(&mut self, _: LexerToken) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod annotation_processing {
    use crate::{lex_with_processor, DropAnnotationsProcessor, LexerAnnotationProcessor, LexerAnnotationProcessorInfo, LexerToken, TokenType};

    #[test]
    fn drop_annotation() {
        let mut lexer_annotation_processor = DropAnnotationsProcessor::new();

        let result = lex_with_processor(&"@annotation".to_string(), &mut lexer_annotation_processor).unwrap();

        assert_eq!(result, vec![]);
        assert_eq!(
            lexer_annotation_processor.get_dropped_annotations(),
            &vec![LexerToken::new("@annotation".to_string(), TokenType::Annotation, 0, 0)]
        )
    }

    #[test]
    fn drop_line_annotation() {
        let mut lexer_annotation_processor = DropAnnotationsProcessor::new();

        let result = lex_with_processor(&"@@This is a comment".to_string(), &mut lexer_annotation_processor).unwrap();

        assert_eq!(result, vec![]);
        assert_eq!(
            lexer_annotation_processor.get_dropped_annotations(),
            &vec![LexerToken::new("@@This is a comment".to_string(), TokenType::LineAnnotation, 0, 0)]
        )
    }

    #[test]
    fn request_until_token_type() {
        struct Processor {
            tokens: Vec<LexerToken>,
        }

        impl LexerAnnotationProcessor for Processor {
            type Error = String;

            fn yield_annotation(&mut self, _: &String, _: usize, _: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
                Ok(LexerAnnotationProcessorInfo::until_token_type(TokenType::Subexpression))
            }

            fn yield_line_annotation(&mut self, _: &String, _: usize, _: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
                todo!()
            }

            fn yield_token(&mut self, token: LexerToken) -> Result<(), Self::Error> {
                self.tokens.push(token.clone());

                Ok(())
            }
        }

        let mut processor = Processor { tokens: vec![] };
        let result = lex_with_processor(&"@some_annotation my_value\n\n500".to_string(), &mut processor).unwrap();

        assert_eq!(result, vec![LexerToken::new("500".to_string(), TokenType::Number, 2, 0)]);
        assert_eq!(
            processor.tokens,
            vec![
                LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 16),
                LexerToken::new("my_value".to_string(), TokenType::Identifier, 0, 17),
                LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 25)
            ]
        )
    }
}
