use crate::{LexerToken, TokenType};

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum LexerAnnotationProcessorInstruction {
    NoOp,
    Drop,
}

pub struct LexerAnnotationProcessorInfo {
    instruction: LexerAnnotationProcessorInstruction,
}

impl LexerAnnotationProcessorInfo {
    pub fn new(instruction: LexerAnnotationProcessorInstruction) -> Self {
        LexerAnnotationProcessorInfo { instruction }
    }

    pub fn get_instruction(&self) -> LexerAnnotationProcessorInstruction {
        self.instruction
    }
}

pub trait LexerAnnotationProcessor {
    type Error: ToString;

    fn yield_annotation(&mut self, annotation: &String, line: usize, column: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error>;
    fn yield_line_annotation(&mut self, annotation: &String, line: usize, column: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error>;
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
        Ok(LexerAnnotationProcessorInfo::new(LexerAnnotationProcessorInstruction::NoOp))
    }

    fn yield_line_annotation(&mut self, _: &String, _: usize, _: usize) -> Result<LexerAnnotationProcessorInfo, Self::Error> {
        Ok(LexerAnnotationProcessorInfo::new(LexerAnnotationProcessorInstruction::NoOp))
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
}

#[cfg(test)]
mod annotation_processing {
    use crate::{lex_with_processor, DropAnnotationsProcessor, LexerToken, TokenType};

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
}
