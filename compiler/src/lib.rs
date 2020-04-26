pub use lexer::*;
pub use classify::*;
pub use ast::*;
pub use build::*;

mod lexer;
mod classify;
mod ast;
mod build;

use garnish_lang_common::Result;
use garnish_lang_instruction_set_builder::InstructionSetBuilder;

pub fn compile(name: &str, input: &str) -> Result<InstructionSetBuilder> {
    let lexed = lexer::Lexer::new().lex(&input)?;
    let parsed = classify::Parser::new().make_groups(&lexed)?;
    let ast = ast::make_ast(parsed)?;
    build::build_byte_code(name, ast)
}
