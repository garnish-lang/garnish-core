mod data_wrapper;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Error, Index, Type, TypeGenerics, WhereClause};

use data_wrapper::process_data_wrapper_proc;

#[proc_macro_attribute]
pub fn delegate_garnish_data(args: TokenStream, item: TokenStream) -> TokenStream {
    process_data_wrapper_proc(args, item)
}

#[proc_macro_derive(GarnishData, attributes(garnish_data))]
pub fn garnish_lang_data_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    #[cfg(feature = "garnish_facade")]
    let library = quote! { garnish_lang };
    #[cfg(not(feature = "garnish_facade"))]
    let library = quote! { garnish_lang_traits };

    let name = &ast.ident;
    let (_, impl_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = match ast.data {
        Data::Struct(data) => match data.fields.len() {
            0 => Err(Error::new_spanned(data.fields, "Expected at least one field in order to derive GarnishData")),
            1 => {
                let first = data.fields.iter().next().unwrap();

                let field_name = match &first.ident {
                    None => Index::from(0).to_token_stream(),
                    Some(ident) => ident.to_token_stream(),
                };

                Ok(create_garnish_data_impl(name, impl_generics, where_clause, field_name, &first.ty, library))
            }
            _ => {
                let marker = data
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_index, field)| field.attrs.iter().find(|a| a.path().is_ident("garnish_data")).is_some());

                match marker {
                    None => Err(Error::new_spanned(
                        name,
                        "In order to derive GarnishData on struct with multiple fields, one field needs to be marked with #[garnish_data]",
                    )),
                    Some((index, field)) => {
                        let field_name = match &field.ident {
                            None => Index::from(index).to_token_stream(),
                            Some(field_name) => field_name.to_token_stream(),
                        };

                        Ok(create_garnish_data_impl(name, impl_generics, where_clause, field_name, &field.ty, library))
                    }
                }
            }
        },
        _ => Err(Error::new_spanned(name, "Deriving GarnishData only supported on structs")),
    };

    expanded.unwrap_or_else(Error::into_compile_error).into()
}

fn create_garnish_data_impl(
    for_type: &Ident,
    impl_generics: TypeGenerics,
    where_clause: Option<&WhereClause>,
    delegate_field: proc_macro2::TokenStream,
    delegate_field_type: &Type,
    library: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics GarnishData for #for_type #impl_generics #where_clause {
            type Error = <#delegate_field_type as GarnishData>::Error;
            type Symbol = <#delegate_field_type as GarnishData>::Symbol;
            type Byte = <#delegate_field_type as GarnishData>::Byte;
            type Char = <#delegate_field_type as GarnishData>::Char;
            type Number = <#delegate_field_type as GarnishData>::Number;
            type Size = <#delegate_field_type as GarnishData>::Size;
            type SizeIterator = <#delegate_field_type as GarnishData>::SizeIterator;
            type NumberIterator = <#delegate_field_type as GarnishData>::NumberIterator;
            type InstructionIterator = <#delegate_field_type as GarnishData>::InstructionIterator;
            type DataIndexIterator = <#delegate_field_type as GarnishData>::DataIndexIterator;
            type ValueIndexIterator = <#delegate_field_type as GarnishData>::ValueIndexIterator;
            type RegisterIndexIterator = <#delegate_field_type as GarnishData>::RegisterIndexIterator;
            type JumpTableIndexIterator = <#delegate_field_type as GarnishData>::JumpTableIndexIterator;
            type JumpPathIndexIterator = <#delegate_field_type as GarnishData>::JumpPathIndexIterator;
            type ListIndexIterator = <#delegate_field_type as GarnishData>::ListIndexIterator;
            type ListItemIterator = <#delegate_field_type as GarnishData>::ListItemIterator;
            type ConcatenationItemIterator = <#delegate_field_type as GarnishData>::ConcatenationItemIterator;
            type CharIterator = <#delegate_field_type as GarnishData>::CharIterator;
            type ByteIterator = <#delegate_field_type as GarnishData>::ByteIterator;
            type SymbolListPartIterator = <#delegate_field_type as GarnishData>::SymbolListPartIterator;
            type DataFactory = <#delegate_field_type as GarnishData>::DataFactory;

            fn get_data_len(&self) -> Self::Size {
                self.#delegate_field.get_data_len()
            }
            fn get_data_iter(&self) -> Self::DataIndexIterator {
                self.#delegate_field.get_data_iter()
            }
            fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_value_stack(addr)
            }
            fn pop_value_stack(&mut self) -> Option<Self::Size> {
                self.#delegate_field.pop_value_stack()
            }
            fn get_current_value(&self) -> Option<Self::Size> {
                self.#delegate_field.get_current_value()
            }
            fn get_current_value_mut(&mut self) -> Option<&mut Self::Size> {
                self.#delegate_field.get_current_value_mut()
            }
            fn get_data_type(&self, addr: Self::Size) -> Result<#library::GarnishDataType, Self::Error> {
                self.#delegate_field.get_data_type(addr)
            }
            fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
                self.#delegate_field.get_number(addr)
            }
            fn get_type(&self, addr: Self::Size) -> Result<#library::GarnishDataType, Self::Error> {
                self.#delegate_field.get_type(addr)
            }
            fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
                self.#delegate_field.get_char(addr)
            }
            fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
                self.#delegate_field.get_byte(addr)
            }
            fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
                self.#delegate_field.get_symbol(addr)
            }
            fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_expression(addr)
            }
            fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_external(addr)
            }
            fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_pair(addr)
            }
            fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_concatenation(addr)
            }
            fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_range(addr)
            }
            fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_slice(addr)
            }
            fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_partial(addr)
            }
            fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_list_len(addr)
            }
            fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.get_list_item(list_addr, item_addr)
            }
            fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.get_list_item_with_symbol(list_addr, sym)
            }
            fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_char_list_len(addr)
            }
            fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Char>, Self::Error> {
                self.#delegate_field.get_char_list_item(addr, item_index)
            }
            fn get_char_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::CharIterator, Self::Error> {
                self.#delegate_field.get_char_list_iter(list_addr, extents)
            }
            fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_byte_list_len(addr)
            }
            fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Byte>, Self::Error> {
                self.#delegate_field.get_byte_list_item(addr, item_index)
            }
            fn get_byte_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ByteIterator, Self::Error> {
                self.#delegate_field.get_byte_list_iter(list_addr, extents)
            }
            fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_symbol_list_len(addr)
            }
            fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<SymbolListPart<Self::Symbol, Self::Number>>, Self::Error> {
                self.#delegate_field.get_symbol_list_item(addr, item_index)
            }
            fn get_symbol_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::SymbolListPartIterator, Self::Error> {
                self.#delegate_field.get_symbol_list_iter(list_addr, extents)
            }
            fn get_list_item_iter(&self, addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListItemIterator, Self::Error> {
                self.#delegate_field.get_list_item_iter(addr, extents)
            }
            fn get_concatenation_iter(&self, addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ConcatenationItemIterator, Self::Error> {
                self.#delegate_field.get_concatenation_iter(addr, extents)
            }
            fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_unit()
            }
            fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_true()
            }
            fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_false()
            }
            fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_number(value)
            }
            fn add_type(&mut self, value: #library::GarnishDataType) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_type(value)
            }
            fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_char(value)
            }
            fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_byte(value)
            }
            fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_symbol(value)
            }
            fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_expression(value)
            }
            fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_external(value)
            }
            fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_pair(value)
            }
            fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_concatenation(left, right)
            }
            fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_range(start, end)
            }
            fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_slice(list, range)
            }
            fn add_partial(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_partial(list, range)
            }
            fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.merge_to_symbol_list(first, second)
            }
            fn start_list(&mut self, len: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.start_list(len)
            }
            fn add_to_list(&mut self, list_index: Self::Size, item_index: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_to_list(list_index, item_index)
            }
            fn end_list(&mut self, list_index: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.end_list(list_index)
            }
            fn get_register_len(&self) -> Self::Size {
                self.#delegate_field.get_register_len()
            }
            fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_register(addr)
            }
            fn get_register(&self, addr: Self::Size) -> Option<Self::Size> {
                self.#delegate_field.get_register(addr)
            }
            fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.pop_register()
            }
            fn get_instruction_len(&self) -> Self::Size {
                self.#delegate_field.get_instruction_len()
            }
            fn push_instruction(&mut self, instruction: #library::Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.push_instruction(instruction, data)
            }
            fn get_instruction(&self, addr: Self::Size) -> Option<(#library::Instruction, Option<Self::Size>)> {
                self.#delegate_field.get_instruction(addr)
            }
            fn get_instruction_iter(&self) -> Self::InstructionIterator {
                self.#delegate_field.get_instruction_iter()
            }
            fn get_instruction_cursor(&self) -> Self::Size {
                self.#delegate_field.get_instruction_cursor()
            }
            fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.set_instruction_cursor(addr)
            }
            fn get_jump_table_len(&self) -> Self::Size {
                self.#delegate_field.get_jump_table_len()
            }
            fn push_to_jump_table(&mut self, index: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_to_jump_table(index)
            }
            fn get_from_jump_table(&self, index: Self::Size) -> Option<Self::Size> {
                self.#delegate_field.get_from_jump_table(index)
            }
            fn get_from_jump_table_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size> {
                self.#delegate_field.get_from_jump_table_mut(index)
            }
            fn push_frame(&mut self, index: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_frame(index)
            }
            fn pop_frame(&mut self) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.pop_frame()
            }
            fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_char_list_from(from)
            }
            fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_byte_list_from(from)
            }
            fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_symbol_from(from)
            }
            fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_number_from(from)
            }
            fn parse_add_number(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_number(from)
            }
            fn parse_add_symbol(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_symbol(from)
            }
            fn parse_add_char(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_char(from)
            }
            fn parse_add_byte(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_byte(from)
            }
            fn parse_add_char_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_char_list(from)
            }
            fn parse_add_byte_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_byte_list(from)
            }
            fn resolve(&mut self, symbol: Self::Symbol) -> Result<bool, Self::Error> {
                self.#delegate_field.resolve(symbol)
            }
            fn apply(&mut self, external_value: Self::Size, input_addr: Self::Size) -> Result<bool, Self::Error> {
                self.#delegate_field.apply(external_value, input_addr)
            }
            fn defer_op(
                &mut self,
                operation: #library::Instruction,
                left: (#library::GarnishDataType, Self::Size),
                right: (#library::GarnishDataType, Self::Size),
            ) -> Result<bool, Self::Error> {
                self.#delegate_field.defer_op(operation, left, right)
            }
        }
    }
}
