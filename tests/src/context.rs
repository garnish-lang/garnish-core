use garnish_lang::GarnishContext;
use garnish_lang::simple::SimpleGarnishData;

pub struct TestingContext {

}

impl Default for TestingContext {
    fn default() -> Self {
        TestingContext {}
    }
}

impl GarnishContext<SimpleGarnishData> for TestingContext {

}