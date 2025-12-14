use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::parse::Parse;
use syn::{Error, ImplItem, ItemImpl, Token, Type, parse_macro_input, parse_quote};

struct GarnishWrapperArgs {
    delegate_field_name: Ident,
    delegate_field_type: Type,
}

impl Parse for GarnishWrapperArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut field_name = None;
        let mut field_type = None;

        let name = input.parse::<Ident>()?;
        if name.to_string() == "delegate_field" {
            input.parse::<Token![=]>()?;
            field_name = Some(input.parse::<Ident>()?);
            let _ = input.parse::<Token![,]>();
        } else if name.to_string() == "delegate_field_type" {
            input.parse::<Token![=]>()?;
            field_type = Some(input.parse::<Type>()?);
            let _ = input.parse::<Token![,]>();
        } else {
            Err(Error::new(name.span(), "Unexpected property."))?
        }

        let name = input.parse::<Ident>()?;
        if name.to_string() == "delegate_field" {
            input.parse::<Token![=]>()?;
            field_name = Some(input.parse::<Ident>()?);
        } else if name.to_string() == "delegate_field_type" {
            input.parse::<Token![=]>()?;
            field_type = Some(input.parse::<Type>()?);
        } else {
            Err(Error::new(name.span(), "Unexpected property."))?
        }

        match (field_name, field_type) {
            (Some(field_name), Some(field_type)) => Ok(GarnishWrapperArgs {
                delegate_field_name: field_name,
                delegate_field_type: field_type,
            }),
            (None, Some(_)) => Err(Error::new(proc_macro2::Span::call_site(), "Expected required property delegate_field.")),
            (Some(_), None) => Err(Error::new(
                proc_macro2::Span::call_site(),
                "Expected required property delegate_field_type.",
            )),
            (None, None) => Err(Error::new(
                proc_macro2::Span::call_site(),
                "Expected required property delegate_field and delegate_field_type.",
            )),
        }
    }
}

pub fn process_data_wrapper_proc(args: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as GarnishWrapperArgs);
    let delegate_field_name = &parsed_args.delegate_field_name;
    let delegate_field_type = &parsed_args.delegate_field_type;

    let mut impl_block = parse_macro_input!(item as ItemImpl);
    let mut implemented_functions: HashSet<String> = HashSet::new();

    for item in impl_block.items.iter() {
        match item {
            ImplItem::Fn(function) => {
                implemented_functions.insert(function.sig.ident.to_string());
            }
            _ => {}
        }
    }

    let associated_types = create_associated_types(delegate_field_type);
    let remaining_items = match create_missing_functions(delegate_field_name, delegate_field_type, &implemented_functions) {
        Ok(items) => items,
        Err(err) => return err.to_compile_error().into(),
    };

    impl_block.items.splice(0..0, remaining_items);
    impl_block.items.splice(0..0, associated_types);

    TokenStream::from(quote! {
        #impl_block
    })
}

fn create_associated_types(delegate_field_type: &Type) -> Vec<ImplItem> {
    let types = vec![
        "Error",
        "Symbol",
        "Byte",
        "Char",
        "Number",
        "Size",
        "SizeIterator",
        "NumberIterator",
        "InstructionIterator",
        "DataIndexIterator",
        "ValueIndexIterator",
        "RegisterIndexIterator",
        "JumpTableIndexIterator",
        "JumpPathIndexIterator",
        "ListIndexIterator",
        "ListItemIterator",
        "ConcatenationItemIterator",
        "CharIterator",
        "ByteIterator",
        "SymbolListPartIterator",
    ];

    let associated_types = types.iter().map(|t| Ident::new(t, proc_macro2::Span::call_site())).map(|ident| {
        ImplItem::Type(parse_quote! {
            type #ident = <#delegate_field_type as GarnishData>::#ident;
        })
    });

    associated_types.collect()
}

fn create_missing_functions(
    delegate_field: &Ident,
    delegate_field_type: &Type,
    implemented_functions: &HashSet<String>,
) -> syn::Result<Vec<ImplItem>> {
    let mut all_functions: HashMap<&str, proc_macro2::TokenStream> = HashMap::new();

    all_functions.insert(
        "get_data_len",
        quote! {
            fn get_data_len(&self) -> Self::Size {
                self.#delegate_field.get_data_len()
            }
        },
    );
    all_functions.insert(
        "get_data_iter",
        quote! {
            fn get_data_iter(&self) -> Self::DataIndexIterator {
                self.#delegate_field.get_data_iter()
            }
        },
    );
    all_functions.insert(
        "get_value_stack_len",
        quote! {
            fn get_value_stack_len(&self) -> Self::Size {
                self.#delegate_field.get_value_stack_len()
            }
        },
    );
    all_functions.insert(
        "push_value_stack",
        quote! {
            fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_value_stack(addr)
            }
        },
    );
    all_functions.insert(
        "pop_value_stack",
        quote! {
            fn pop_value_stack(&mut self) -> Option<Self::Size> {
                self.#delegate_field.pop_value_stack()
            }
        },
    );
    all_functions.insert(
        "get_value",
        quote! {
            fn get_value(&self, addr: Self::Size) -> Option<Self::Size> {
                self.#delegate_field.get_value(addr)
            }
        },
    );
    all_functions.insert(
        "get_value_mut",
        quote! {
            fn get_value_mut(&mut self, addr: Self::Size) -> Option<&mut Self::Size> {
                self.#delegate_field.get_value_mut(addr)
            }
        },
    );
    all_functions.insert(
        "get_current_value",
        quote! {
            fn get_current_value(&self) -> Option<Self::Size> {
                self.#delegate_field.get_current_value()
            }
        },
    );
    all_functions.insert(
        "get_current_value_mut",
        quote! {
            fn get_current_value_mut(&mut self) -> Option<&mut Self::Size> {
                self.#delegate_field.get_current_value_mut()
            }
        },
    );
    all_functions.insert(
        "get_value_iter",
        quote! {
            fn get_value_iter(&self) -> Self::ValueIndexIterator {
                self.#delegate_field.get_value_iter()
            }
        },
    );
    all_functions.insert(
        "get_data_type",
        quote! {
            fn get_data_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
                self.#delegate_field.get_data_type(addr)
            }
        },
    );
    all_functions.insert(
        "get_number",
        quote! {
            fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
                self.#delegate_field.get_number(addr)
            }
        },
    );
    all_functions.insert(
        "get_type",
        quote! {
            fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
                self.#delegate_field.get_type(addr)
            }
        },
    );
    all_functions.insert(
        "get_char",
        quote! {
            fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
                self.#delegate_field.get_char(addr)
            }
        },
    );
    all_functions.insert(
        "get_byte",
        quote! {
            fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
                self.#delegate_field.get_byte(addr)
            }
        },
    );
    all_functions.insert(
        "get_symbol",
        quote! {
            fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
                self.#delegate_field.get_symbol(addr)
            }
        },
    );
    all_functions.insert(
        "get_expression",
        quote! {
            fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_expression(addr)
            }
        },
    );
    all_functions.insert(
        "get_external",
        quote! {
            fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_external(addr)
            }
        },
    );
    all_functions.insert(
        "get_pair",
        quote! {
            fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_pair(addr)
            }
        },
    );
    all_functions.insert(
        "get_concatenation",
        quote! {
            fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_concatenation(addr)
            }
        },
    );
    all_functions.insert(
        "get_range",
        quote! {
            fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_range(addr)
            }
        },
    );
    all_functions.insert(
        "get_slice",
        quote! {
            fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_slice(addr)
            }
        },
    );
    all_functions.insert(
        "get_partial",
        quote! {
            fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
                self.#delegate_field.get_partial(addr)
            }
        },
    );
    all_functions.insert(
        "get_list_len",
        quote! {
            fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_list_len(addr)
            }
        },
    );
    all_functions.insert(
        "get_list_item",
        quote! {
            fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.get_list_item(list_addr, item_addr)
            }
        },
    );
    all_functions.insert(
        "get_list_associations_len",
        quote! {
            fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_list_associations_len(addr)
            }
        },
    );
    all_functions.insert(
        "get_list_association",
        quote! {
            fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.get_list_association(list_addr, item_addr)
            }
        },
    );
    all_functions.insert(
        "get_list_item_with_symbol",
        quote! {
            fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.get_list_item_with_symbol(list_addr, sym)
            }
        },
    );
    all_functions.insert(
        "get_list_items_iter",
        quote! {
            fn get_list_items_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListIndexIterator, Self::Error> {
                self.#delegate_field.get_list_items_iter(list_addr, extents)
            }
        },
    );
    all_functions.insert(
        "get_list_associations_iter",
        quote! {
            fn get_list_associations_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListIndexIterator, Self::Error> {
                self.#delegate_field.get_list_associations_iter(list_addr, extents)
            }
        },
    );
    all_functions.insert(
        "get_char_list_len",
        quote! {
            fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_char_list_len(addr)
            }
        },
    );
    all_functions.insert(
        "get_char_list_item",
        quote! {
            fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Char>, Self::Error> {
                self.#delegate_field.get_char_list_item(addr, item_index)
            }
        },
    );
    all_functions.insert(
        "get_char_list_iter",
        quote! {
            fn get_char_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::CharIterator, Self::Error> {
                self.#delegate_field.get_char_list_iter(list_addr, extents)
            }
        },
    );
    all_functions.insert(
        "get_byte_list_len",
        quote! {
            fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_byte_list_len(addr)
            }
        },
    );
    all_functions.insert(
        "get_byte_list_item",
        quote! {
            fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Byte>, Self::Error> {
                self.#delegate_field.get_byte_list_item(addr, item_index)
            }
        },
    );
    all_functions.insert(
        "get_byte_list_iter",
        quote! {
            fn get_byte_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ByteIterator, Self::Error> {
                self.#delegate_field.get_byte_list_iter(list_addr, extents)
            }
        },
    );
    all_functions.insert(
        "get_symbol_list_len",
        quote! {
            fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.get_symbol_list_len(addr)
            }
        },
    );
    all_functions.insert(
        "get_symbol_list_item",
        quote! {
            fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<SymbolListPart<Self::Symbol, Self::Number>>, Self::Error> {
                self.#delegate_field.get_symbol_list_item(addr, item_index)
            }
        },
    );
    all_functions.insert(
        "get_symbol_list_iter",
        quote! {
            fn get_symbol_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::SymbolListPartIterator, Self::Error> {
                self.#delegate_field.get_symbol_list_iter(list_addr, extents)
            }
        },
    );
    all_functions.insert(
        "get_list_item_iter",
        quote! {
            fn get_list_item_iter(&self, addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListItemIterator, Self::Error> {
                self.#delegate_field.get_list_item_iter(addr, extents)
            }
        },
    );
    all_functions.insert(
        "get_concatenation_iter",
        quote! {
            fn get_concatenation_iter(&self, addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ConcatenationItemIterator, Self::Error> {
                self.#delegate_field.get_concatenation_iter(addr, extents)
            }
        },
    );
    all_functions.insert(
        "add_unit",
        quote! {
            fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_unit()
            }
        },
    );
    all_functions.insert(
        "add_true",
        quote! {
            fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_true()
            }
        },
    );
    all_functions.insert(
        "add_false",
        quote! {
            fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_false()
            }
        },
    );
    all_functions.insert(
        "add_number",
        quote! {
            fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_number(value)
            }
        },
    );
    all_functions.insert(
        "add_type",
        quote! {
            fn add_type(&mut self, value: GarnishDataType) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_type(value)
            }
        },
    );
    all_functions.insert(
        "add_char",
        quote! {
            fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_char(value)
            }
        },
    );
    all_functions.insert(
        "add_byte",
        quote! {
            fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_byte(value)
            }
        },
    );
    all_functions.insert(
        "add_symbol",
        quote! {
            fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_symbol(value)
            }
        },
    );
    all_functions.insert(
        "add_expression",
        quote! {
            fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_expression(value)
            }
        },
    );
    all_functions.insert(
        "add_external",
        quote! {
            fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_external(value)
            }
        },
    );
    all_functions.insert(
        "add_pair",
        quote! {
            fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_pair(value)
            }
        },
    );
    all_functions.insert(
        "add_concatenation",
        quote! {
            fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_concatenation(left, right)
            }
        },
    );
    all_functions.insert(
        "add_range",
        quote! {
            fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_range(start, end)
            }
        },
    );
    all_functions.insert(
        "add_slice",
        quote! {
            fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_slice(list, range)
            }
        },
    );
    all_functions.insert(
        "add_partial",
        quote! {
            fn add_partial(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_partial(list, range)
            }
        },
    );
    all_functions.insert(
        "merge_to_symbol_list",
        quote! {
            fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.merge_to_symbol_list(first, second)
            }
        },
    );
    all_functions.insert(
        "start_list",
        quote! {
            fn start_list(&mut self, len: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.start_list(len)
            }
        },
    );
    all_functions.insert(
        "add_to_list",
        quote! {
            fn add_to_list(&mut self, addr: Self::Size, is_associative: bool) -> Result<(), Self::Error> {
                self.#delegate_field.add_to_list(addr, is_associative)
            }
        },
    );
    all_functions.insert(
        "end_list",
        quote! {
            fn end_list(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.end_list()
            }
        },
    );
    all_functions.insert(
        "start_char_list",
        quote! {
            fn start_char_list(&mut self) -> Result<(), Self::Error> {
                self.#delegate_field.start_char_list()
            }
        },
    );
    all_functions.insert(
        "add_to_char_list",
        quote! {
            fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error> {
                self.#delegate_field.add_to_char_list(c)
            }
        },
    );
    all_functions.insert(
        "end_char_list",
        quote! {
            fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.end_char_list()
            }
        },
    );
    all_functions.insert(
        "start_byte_list",
        quote! {
            fn start_byte_list(&mut self) -> Result<(), Self::Error> {
                self.#delegate_field.start_byte_list()
            }
        },
    );
    all_functions.insert(
        "add_to_byte_list",
        quote! {
            fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error> {
                self.#delegate_field.add_to_byte_list(c)
            }
        },
    );
    all_functions.insert(
        "end_byte_list",
        quote! {
            fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.end_byte_list()
            }
        },
    );
    all_functions.insert(
        "get_register_len",
        quote! {
            fn get_register_len(&self) -> Self::Size {
                self.#delegate_field.get_register_len()
            }
        },
    );
    all_functions.insert(
        "push_register",
        quote! {
            fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_register(addr)
            }
        },
    );
    all_functions.insert(
        "get_register",
        quote! {
            fn get_register(&self, addr: Self::Size) -> Option<Self::Size> {
                self.#delegate_field.get_register(addr)
            }
        },
    );
    all_functions.insert(
        "pop_register",
        quote! {
            fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
                self.#delegate_field.pop_register()
            }
        },
    );
    all_functions.insert(
        "get_register_iter",
        quote! {
            fn get_register_iter(&self) -> Self::RegisterIndexIterator {
                self.#delegate_field.get_register_iter()
            }
        },
    );
    all_functions.insert(
        "get_instruction_len",
        quote! {
            fn get_instruction_len(&self) -> Self::Size {
                self.#delegate_field.get_instruction_len()
            }
        },
    );
    all_functions.insert(
        "push_instruction",
        quote! {
            fn push_instruction(&mut self, instruction: Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.push_instruction(instruction, data)
            }
        },
    );
    all_functions.insert(
        "get_instruction",
        quote! {
            fn get_instruction(&self, addr: Self::Size) -> Option<(Instruction, Option<Self::Size>)> {
                self.#delegate_field.get_instruction(addr)
            }
        },
    );
    all_functions.insert(
        "get_instruction_iter",
        quote! {
            fn get_instruction_iter(&self) -> Self::InstructionIterator {
                self.#delegate_field.get_instruction_iter()
            }
        },
    );
    all_functions.insert(
        "get_instruction_cursor",
        quote! {
            fn get_instruction_cursor(&self) -> Self::Size {
                self.#delegate_field.get_instruction_cursor()
            }
        },
    );
    all_functions.insert(
        "set_instruction_cursor",
        quote! {
            fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.set_instruction_cursor(addr)
            }
        },
    );
    all_functions.insert(
        "get_jump_table_len",
        quote! {
            fn get_jump_table_len(&self) -> Self::Size {
                self.#delegate_field.get_jump_table_len()
            }
        },
    );
    all_functions.insert(
        "push_jump_point",
        quote! {
            fn push_jump_point(&mut self, index: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_jump_point(index)
            }
        },
    );
    all_functions.insert(
        "get_jump_point",
        quote! {
            fn get_jump_point(&self, index: Self::Size) -> Option<Self::Size> {
                self.#delegate_field.get_jump_point(index)
            }
        },
    );
    all_functions.insert(
        "get_jump_point_mut",
        quote! {
            fn get_jump_point_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size> {
                self.#delegate_field.get_jump_point_mut(index)
            }
        },
    );
    all_functions.insert(
        "get_jump_table_iter",
        quote! {
            fn get_jump_table_iter(&self) -> Self::JumpTableIndexIterator {
                self.#delegate_field.get_jump_table_iter()
            }
        },
    );
    all_functions.insert(
        "push_jump_path",
        quote! {
            fn push_jump_path(&mut self, index: Self::Size) -> Result<(), Self::Error> {
                self.#delegate_field.push_jump_path(index)
            }
        },
    );
    all_functions.insert(
        "pop_jump_path",
        quote! {
            fn pop_jump_path(&mut self) -> Option<Self::Size> {
                self.#delegate_field.pop_jump_path()
            }
        },
    );
    all_functions.insert(
        "get_jump_path_iter",
        quote! {
            fn get_jump_path_iter(&self) -> Self::JumpPathIndexIterator {
                self.#delegate_field.get_jump_path_iter()
            }
        },
    );
    all_functions.insert(
        "size_to_number",
        quote! {
            fn size_to_number(from: Self::Size) -> Self::Number {
                <#delegate_field_type as GarnishData>::size_to_number(from)
            }
        },
    );
    all_functions.insert(
        "number_to_size",
        quote! {
            fn number_to_size(from: Self::Number) -> Option<Self::Size> {
                <#delegate_field_type as GarnishData>::number_to_size(from)
            }
        },
    );
    all_functions.insert(
        "number_to_char",
        quote! {
            fn number_to_char(from: Self::Number) -> Option<Self::Char> {
                <#delegate_field_type as GarnishData>::number_to_char(from)
            }
        },
    );
    all_functions.insert(
        "number_to_byte",
        quote! {
            fn number_to_byte(from: Self::Number) -> Option<Self::Byte> {
                <#delegate_field_type as GarnishData>::number_to_byte(from)
            }
        },
    );
    all_functions.insert(
        "char_to_number",
        quote! {
            fn char_to_number(from: Self::Char) -> Option<Self::Number> {
                <#delegate_field_type as GarnishData>::char_to_number(from)
            }
        },
    );
    all_functions.insert(
        "char_to_byte",
        quote! {
            fn char_to_byte(from: Self::Char) -> Option<Self::Byte> {
                <#delegate_field_type as GarnishData>::char_to_byte(from)
            }
        },
    );
    all_functions.insert(
        "byte_to_number",
        quote! {
            fn byte_to_number(from: Self::Byte) -> Option<Self::Number> {
                <#delegate_field_type as GarnishData>::byte_to_number(from)
            }
        },
    );
    all_functions.insert(
        "byte_to_char",
        quote! {
            fn byte_to_char(from: Self::Byte) -> Option<Self::Char> {
                <#delegate_field_type as GarnishData>::byte_to_char(from)
            }
        },
    );
    all_functions.insert(
        "add_char_list_from",
        quote! {
            fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_char_list_from(from)
            }
        },
    );
    all_functions.insert(
        "add_byte_list_from",
        quote! {
            fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_byte_list_from(from)
            }
        },
    );
    all_functions.insert(
        "add_symbol_from",
        quote! {
            fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_symbol_from(from)
            }
        },
    );
    all_functions.insert(
        "add_byte_from",
        quote! {
            fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_byte_from(from)
            }
        },
    );
    all_functions.insert(
        "add_number_from",
        quote! {
            fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.add_number_from(from)
            }
        },
    );
    all_functions.insert(
        "parse_number",
        quote! {
            fn parse_number(from: &str) -> Result<Self::Number, Self::Error> {
                <#delegate_field_type as GarnishData>::parse_number(from)
            }
        },
    );
    all_functions.insert(
        "parse_symbol",
        quote! {
            fn parse_symbol(from: &str) -> Result<Self::Symbol, Self::Error> {
                <#delegate_field_type as GarnishData>::parse_symbol(from)
            }
        },
    );
    all_functions.insert(
        "parse_char",
        quote! {
            fn parse_char(from: &str) -> Result<Self::Char, Self::Error> {
                <#delegate_field_type as GarnishData>::parse_char(from)
            }
        },
    );
    all_functions.insert(
        "parse_byte",
        quote! {
            fn parse_byte(from: &str) -> Result<Self::Byte, Self::Error> {
                <#delegate_field_type as GarnishData>::parse_byte(from)
            }
        },
    );
    all_functions.insert(
        "parse_char_list",
        quote! {
            fn parse_char_list(from: &str) -> Result<Vec<Self::Char>, Self::Error> {
                <#delegate_field_type as GarnishData>::parse_char_list(from)
            }
        },
    );
    all_functions.insert(
        "parse_byte_list",
        quote! {
            fn parse_byte_list(from: &str) -> Result<Vec<Self::Byte>, Self::Error> {
                <#delegate_field_type as GarnishData>::parse_byte_list(from)
            }
        },
    );
    all_functions.insert(
        "parse_add_number",
        quote! {
            fn parse_add_number(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_number(from)
            }
        },
    );
    all_functions.insert(
        "parse_add_symbol",
        quote! {
            fn parse_add_symbol(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_symbol(from)
            }
        },
    );
    all_functions.insert(
        "parse_add_char",
        quote! {
            fn parse_add_char(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_char(from)
            }
        },
    );
    all_functions.insert(
        "parse_add_byte",
        quote! {
            fn parse_add_byte(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_byte(from)
            }
        },
    );
    all_functions.insert(
        "parse_add_char_list",
        quote! {
            fn parse_add_char_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_char_list(from)
            }
        },
    );
    all_functions.insert(
        "parse_add_byte_list",
        quote! {
            fn parse_add_byte_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
                self.#delegate_field.parse_add_byte_list(from)
            }
        },
    );
    all_functions.insert(
        "make_size_iterator_range",
        quote! {
            fn make_size_iterator_range(min: Self::Size, max: Self::Size) -> Self::SizeIterator {
                <#delegate_field_type as GarnishData>::make_size_iterator_range(min, max)
            }
        },
    );
    all_functions.insert(
        "make_number_iterator_range",
        quote! {
            fn make_number_iterator_range(min: Self::Number, max: Self::Number) -> Self::NumberIterator {
                <#delegate_field_type as GarnishData>::make_number_iterator_range(min, max)
            }
        },
    );

    all_functions.insert(
        "resolve",
        quote! {
            fn resolve(&mut self, symbol: Self::Symbol) -> Result<bool, Self::Error> {
                self.#delegate_field.resolve(symbol)
            }
        },
    );
    all_functions.insert(
        "apply",
        quote! {
            fn apply(&mut self, external_value: Self::Size, input_addr: Self::Size) -> Result<bool, Self::Error> {
                self.#delegate_field.apply(external_value, input_addr)
            }
        },
    );
    all_functions.insert(
        "defer_op",
        quote! {
            fn defer_op(
                &mut self,
                operation: Instruction,
                left: (GarnishDataType, Self::Size),
                right: (GarnishDataType, Self::Size),
            ) -> Result<bool, Self::Error> {
                self.#delegate_field.defer_op(operation, left, right)
            }
        },
    );

    let mut missing_items = Vec::new();
    for (name, item_tokens) in all_functions {
        if !implemented_functions.contains(name) {
            missing_items.push(syn::parse2::<ImplItem>(item_tokens.clone())?);
        }
    }

    Ok(missing_items)
}
