use std::fmt::Debug;
use std::hash::Hash;
use garnish_lang::simple::{SimpleDataType, SimpleGarnishData};
use garnish_lang::{GarnishData, SymbolListPart};

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct Data {

}

impl SimpleDataType for Data {}

#[derive(GarnishData)]
pub struct GarnishDataDerivedWithGenerics<T> where T: SimpleDataType {
    data: SimpleGarnishData<T>
}

#[derive(GarnishData)]
pub struct GarnishDataDerivedCustomData {
    data: SimpleGarnishData<Data>
}

#[derive(GarnishData)]
pub struct GarnishDataDerived {
    data: SimpleGarnishData<Data>
}

#[derive(GarnishData)]
pub struct GarnishDataDerivedWithOtherMarkerProp {
    _name: String,
    #[garnish_data]
    data: SimpleGarnishData<Data>
}

#[derive(GarnishData)]
pub struct GarnishDataDerivedTuple(SimpleGarnishData<Data>);

#[derive(GarnishData)]
pub struct GarnishDataDerivedTupleWithMarker((), #[garnish_data] SimpleGarnishData<Data>);

// errors, uncomment to see error

// #[derive(GarnishData)]
// pub struct GarnishDataDerivedWithNoMarkerProp {
//     _name: String,
//     data: SimpleGarnishData
// }

// #[derive(GarnishData)]
// pub struct GarnishDataDerivedTupleWithNoMarker((), SimpleGarnishData);