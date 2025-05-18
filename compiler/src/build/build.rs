use crate::error::CompilerError;
use crate::parse::ParseNode;
use garnish_lang_traits::GarnishData;

pub struct BuildData {
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
}

pub fn build<Data: GarnishData>(parse_root: usize, parse_tree: Vec<ParseNode>, data: &mut Data) -> Result<BuildData, CompilerError<Data::Error>> {
    Ok(BuildData { parse_root, parse_tree })
}

#[cfg(test)]
mod tests {
    use garnish_lang_simple_data::SimpleGarnishData;
    use crate::build::build::build;

    #[test]
    fn build_empty() {
        let mut data = SimpleGarnishData::new();
        build(0, vec![], &mut data).unwrap();
        
        assert!(data.get_instructions().is_empty());
    }
}
