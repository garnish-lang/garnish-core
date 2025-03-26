use garnish_lang_simple_data::SimpleGarnishData;
use garnish_lang_traits::GarnishContext;

pub struct TestingContext {

}

impl Default for TestingContext {
    fn default() -> Self {
        TestingContext {}
    }
}

impl GarnishContext<SimpleGarnishData> for TestingContext {

}