use std::fmt::Debug;
use std::hash::Hash;
use garnish_lang::simple::SimpleGarnishData;
use garnish_lang::GarnishData;

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct Data {

}

#[derive(GarnishData)]
pub struct GarnishDataWrapperWithGenerics<T> where T: Debug + Clone + PartialEq + Eq + Hash + PartialOrd {
    data: SimpleGarnishData<T>
}

#[derive(GarnishData)]
pub struct GarnishDataWrapperCustomData {
    data: SimpleGarnishData<Data>
}

#[derive(GarnishData)]
pub struct GarnishDataWrapper {
    data: SimpleGarnishData<Data>
}

#[derive(GarnishData)]
pub struct GarnishDataWrapperWithOtherMarkerProp {
    _name: String,
    #[garnish_data]
    data: SimpleGarnishData<Data>
}

#[derive(GarnishData)]
pub struct GarnishDataWrapperTuple(SimpleGarnishData<Data>);

#[derive(GarnishData)]
pub struct GarnishDataWrapperTupleWithMarker((), #[garnish_data] SimpleGarnishData<Data>);

// errors, uncomment to see error

// #[derive(GarnishData)]
// pub struct GarnishDataWrapperWithNoMarkerProp {
//     _name: String,
//     data: SimpleGarnishData
// }

// #[derive(GarnishData)]
// pub struct GarnishDataWrapperTupleWithNoMarker((), SimpleGarnishData);