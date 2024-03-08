# Garnish Core &emsp; [![Build Status]][actions]

[Build Status]: https://img.shields.io/github/actions/workflow/status/garnish-lang/garnish-core/rust.yml?branch=main&label=Tests
[actions]: https://github.com/garnish-lang/garnish-core/actions?branch=main

Core libraries needed to embed the garnish language. These are what you will need to add Garnish scripting to an application.

If your interested in learning about the Garnish Language, please visit the [Demo Site][demo.site].

[demo.site]: https://garnish-lang.github.io/garnish-site/

## Libraries
This repository contains the following library crates. Version numbers for each are kept in sync with one another even if it means no changes were made to that library.

### Traits
[![Traits Crate]][traits.crates.io] [![Traits Docs]][traits.docsrs]

[Traits Crate]: https://img.shields.io/crates/v/garnish_lang_traits.svg?color=darkgreen
[traits.crates.io]: https://crates.io/crates/garnish_lang_traits

[Traits Docs]: https://img.shields.io/docsrs/garnish_lang_traits.svg?color=cc5500&label=docsrs
[traits.docsrs]: https://docs.rs/garnish_lang_traits/latest/garnish_lang_traits/

Contains base traits, structs, enums, etc. for use by rest of the core libraries.

### Simple Data
[![Simple Data Crate]][simple-data.crates.io] [![Simple Data Docs]][simple-data.docsrs]

[Simple Data Crate]: https://img.shields.io/crates/v/garnish_lang_simple_data.svg?color=darkgreen
[simple-data.crates.io]: https://crates.io/crates/garnish_lang_simple_data

[Simple Data Docs]: https://img.shields.io/docsrs/garnish_lang_simple_data.svg?color=cc5500&label=docsrs
[simple-data.docsrs]: https://docs.rs/garnish_lang_simple_data/latest/garnish_lang_simple_data/

An implementation of `GarnishData` using standard Rust types and data structures.

### Runtime
[![Runtime Crate]][runtime.crates.io] [![Traits Docs]][runtime.docsrs]

[Runtime Crate]: https://img.shields.io/crates/v/garnish_lang_runtime.svg?color=darkgreen
[runtime.crates.io]: https://crates.io/crates/garnish_lang_runtime

[Runtime Docs]: https://img.shields.io/docsrs/garnish_lang_runtime.svg?color=cc5500&label=docsrs
[runtime.docsrs]: https://docs.rs/garnish_lang_runtime/latest/garnish_lang_runtime/

An implementation of `GarnishRuntime` which executes instructions upon given data object.

### Compiler
[![Compiler Crate]][compiler.crates.io] [![Compiler Docs]][compiler.docsrs]

[Compiler Crate]: https://img.shields.io/crates/v/garnish_lang_compiler.svg?color=darkgreen
[compiler.crates.io]: https://crates.io/crates/garnish_lang_compiler

[Compiler Docs]: https://img.shields.io/docsrs/garnish_lang_compiler.svg?color=cc5500&label=docsrs
[compiler.docsrs]: https://docs.rs/garnish_lang_compiler/latest/garnish_lang_compiler/

Contains functions to lex and parse and input string and building that instruction set into a data object.

### Garnish Lang
[![Lang Crate]][lang.crates.io] [![Lang Docs]][lang.docsrs]

[Lang Crate]: https://img.shields.io/crates/v/garnish_lang.svg?color=darkgreen
[lang.crates.io]: https://crates.io/crates/garnish_lang

[Lang Docs]: https://img.shields.io/docsrs/garnish_lang.svg?color=cc5500&label=docsrs
[lang.docsrs]: https://docs.rs/garnish_lang/latest/garnish_lang/

Convenience single dependency for above four libraries.

## Usage
These examples use the [Garnish Lang][lang.crates.io] crate. If you plan to import the four individually, simply adjust the `use` statements accordingly.

### Basic Compile and Execute
With just the core libraries this is a three-step process and a `GarnishData` object will need to be created for the third.

```rust
use garnish_lang::compiler::lex::{lex, LexerToken};
use garnish_lang::compiler::parse::{parse, ParseResult};
use garnish_lang::compiler::build::build_with_data;
use garnish_lang::simple::SimpleGarnishData;

const INPUT: &str = "5 + 5";

fn main() -> Result<(), String> {
    let tokens: Vec<LexerToken> = lex(input).or_else(|e| Err(e.get_message().clone()))?;

    let parse_result: ParseResult = parse(&tokens).or_else(|e| Err(e.get_message().clone()))?;

    let mut data = SimpleGarnishData::new();

    build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data)
        .or_else(|e| Err(e.get_message().clone()))?;
    
    let mut runtime = SimpleGarnishRuntime::new(data);
    
    // SimpleGarnishRuntime only provides method to execute instructions 1 at a time, 
    // so we loop until finished
    loop {
        // this None argument is where a GarnishContext would be passed
        match runtime.execute_current_instruction(None) {
            Err(e) => {
                return Err(e.get_message().clone());
            }
            Ok(data) => match data.get_state() {
                SimpleRuntimeState::Running => (),
                SimpleRuntimeState::End => break,
            },
        }
    }
    
    // Result of an execution is a data objects current value
    runtime.get_data().get_current_value().and_then(|v| {
        // get_raw_data is not a trait member of GarnishData, 
        // but a convenience function of SimpleGarnishData
        println!("Result: {:?}", runtime.get_data().get_raw_data(v))
    });
    
    Ok(())
}
```

### Using Context
Providing a `GarnishContext` object during execution is a way to extend the functionality of a script. 
This can be providing environment variables, methods for accessing a database, or customizing operations.

The following example provides two items to a script. A constant value for PI and a way to execute the trigonometric function sine.

```rust
use std::collections::HashMap;
use garnish_lang::{GarnishContext, GarnishData, RuntimeError};
use garnish_lang::simple::{
    DataError, 
    SimpleData, 
    SimpleGarnishData, 
    SimpleNumber, 
    symbol_value
};

const MATH_FUNCTION_SINE: usize = 1;

pub struct MathContext {
    symbol_to_data: HashMap<u64, SimpleData>
}

impl MathContext {
    pub fn new() -> Self {
        let mut symbol_to_data = HashMap::new();
        symbol_to_data.insert(
            symbol_value("Math::PI"), 
            SimpleData::Number(SimpleNumber::Float(std::f64::consts::PI))
        );
        symbol_to_data.insert(
            symbol_value("sin"), 
            SimpleData::External(MATH_FUNCTION_SINE)
        );

        BrowserContext {
            symbol_to_expression: HashMap::new(),
            symbol_to_data
        }
    }
}

impl GarnishContext<SimpleGarnishData> for MathContext {
    // This method is called when ever a script has an unresolved identifier during runtime
    fn resolve(&mut self, symbol: u64, data: &mut SimpleGarnishData) 
        -> Result<bool, RuntimeError<DataError>> {
        // lookup given symbol to see if we have a value for it
        // returning true tells runtime that the symbol was resolved 
        //   and not to do any more checks
        // returning false will let the runtime check additional resolve methods, 
        //   resulting in a Unit value if nothing resolves it
        match self.symbol_to_data.get(&symbol) {
            Some(v) => match v {
                SimpleData::External(n) => {
                    // using GarnishData trait methods, add_* for each GarnishDataType
                    data.add_external(*n).and_then(|addr| data.push_register(addr))?;
                    Ok(true)
                },
                SimpleData::Number(n) => {
                    data.add_number(*n).and_then(|addr| data.push_register(addr))?;
                    Ok(true)
                },
                _ => Ok(false)
            }
            None => Ok(false)
        }
    }
    
    // This method is called when ever an External type value 
    // is used with Garnish's 'apply' type operations
    fn apply(
        &mut self,
        external_value: usize,
        input_addr: usize,
        data: &mut SimpleGarnishData,
    ) -> Result<bool, RuntimeError<DataError>> {
        // check that the external value given is actually supported
        if external_value == MATH_FUNCTION_SINE {
            // using some non trait methods, whether to use trait methods or 
            // implementation specific methods will depend on your use case
            let new_data = data.get_raw_data(input_addr).and_then(|d| Some(match d {
                SimpleData::Number(num) => SimpleData::Number(SimpleNumber::Float(match num {
                    SimpleNumber::Integer(n) => f64::sin(n as f64),
                    SimpleNumber::Float(f) => f64::sin(f),
                })),
                // sin function only supports numbers, all other values result in Unit
                _ => SimpleData::Unit
            })).ok_or(
                DataError::from("Failed to retrieve data during external apply 'sin'"
                    .to_string())
            )?;

            // need to add new data 
            // then push its address to registers for next operation to use
            // failure to not push expected values and still returning true, 
            //   could cause script to fail due to empty registers
            let addr = data.get_data().len();
            data.get_data_mut().push(new_data);
            data.push_register(addr)?;

            Ok(true)
        } else {
            // return value signifies same as resolve method's return value
            Ok(false)
        }
    }
}
```

Now we've implemented a `GarnishContext` we can pass it into the `execute_current_instruction` method instead of None.

```rust
// ...
let mut runtime = SimpleGarnishRuntime::new(data);
let mut context = MathContext::new();

loop {
    // add new context object
    match runtime.execute_current_instruction(Some(&mut context)) {
        Err(e) => {
            return Err(e.get_message().clone());
        }
        Ok(data) => match data.get_state() {
            SimpleRuntimeState::Running => (),
            SimpleRuntimeState::End => break,
        },
    }
}
// ...
```

### Further Reading
The [Browser Garnish](https://github.com/garnish-lang/browser-garnish) project is the WebAssembly library used by the [Demo Site][demo.site].
Going through the demo and viewing the source will illustrate how it all links together.

[API Documentation][lang.docsrs] - For full descriptions and more examples. (Currently still working in progress)