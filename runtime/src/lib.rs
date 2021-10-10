use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub struct ExpressionData {
    bytes: Vec<u8>
}

impl ExpressionData {
    pub fn integer(i: i64) -> Result<ExpressionData, String> {
        let mut data = ExpressionData { bytes: vec![] };
        match data.bytes.write_i64::<LittleEndian>(i) {
            Ok(_) => Result::Ok(data),
            Err(_) => Err("Could not write integer data.".to_string())
        }
    }
}

pub struct GarnishLangRuntime {
    data: Vec<ExpressionData>
}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        return GarnishLangRuntime {
            data: vec![]
        }
    }

    pub fn add_data(&mut self, data: ExpressionData) -> Result<(), String> {
        self.data.push(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{GarnishLangRuntime, ExpressionData};

    #[test]
    fn create_runtime() {
        GarnishLangRuntime::new();
    }

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100).unwrap()).unwrap();

        assert_eq!(runtime.data.len(), 1);
    }
}