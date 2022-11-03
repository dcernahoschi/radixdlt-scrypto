use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_describe(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_describe() starts");

    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse2(input)?;
    if !generics.params.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "Generics are not presently supported with Describe",
        ));
    }

    let ident_str = ident.to_string();
    trace!("Describing: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_describing_skipped(f)).collect();

                let names = ns.iter().map(|f| {
                    f.ident
                        .clone()
                        .expect("All fields must be named")
                        .to_string()
                });
                let types = ns.iter().map(|f| &f.ty);

                quote! {
                    impl ::scrypto::abi::Describe for #ident {
                        fn describe() -> ::scrypto::abi::Type {
                            use ::sbor::rust::borrow::ToOwned;
                            use ::sbor::rust::vec;
                            use ::scrypto::abi::Describe;

                            ::scrypto::abi::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::scrypto::abi::Fields::Named {
                                    named: vec![#((#names.to_owned(), <#types>::describe())),*]
                                },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns: Vec<&Field> = unnamed
                    .iter()
                    .filter(|f| !is_describing_skipped(f))
                    .collect();

                let types = ns.iter().map(|f| &f.ty);

                quote! {
                    impl ::scrypto::abi::Describe for #ident {
                        fn describe() -> ::scrypto::abi::Type {
                            use ::sbor::rust::borrow::ToOwned;
                            use ::sbor::rust::vec;
                            use ::scrypto::abi::Describe;

                            ::scrypto::abi::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::scrypto::abi::Fields::Unnamed {
                                    unnamed: vec![#(<#types>::describe()),*]
                                },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::scrypto::abi::Describe for #ident {
                        fn describe() -> ::scrypto::abi::Type {
                            use ::sbor::rust::borrow::ToOwned;

                            ::scrypto::abi::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::scrypto::abi::Fields::Unit,
                            }
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let names = variants.iter().map(|v| v.ident.to_string());
            let fields = variants.iter().map(|v| {
                let f = &v.fields;

                match f {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let ns: Vec<&Field> =
                            named.iter().filter(|f| !is_describing_skipped(f)).collect();

                        let names = ns.iter().map(|f| {
                            f.ident
                                .clone()
                                .expect("All fields must be named")
                                .to_string()
                        });
                        let types = ns.iter().map(|f| &f.ty);

                        quote! {
                            {
                                ::scrypto::abi::Fields::Named {
                                    named: vec![#((#names.to_owned(), <#types>::describe())),*]
                                }
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let ns: Vec<&Field> = unnamed
                            .iter()
                            .filter(|f| !is_describing_skipped(f))
                            .collect();

                        let types = ns.iter().map(|f| &f.ty);

                        quote! {
                            {
                                ::scrypto::abi::Fields::Unnamed {
                                    unnamed: vec![#(<#types>::describe()),*]
                                }
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            {
                                ::scrypto::abi::Fields::Unit
                            }
                        }
                    }
                }
            });

            quote! {
                impl ::scrypto::abi::Describe for #ident {
                    fn describe() -> ::scrypto::abi::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::scrypto::abi::Describe;

                        ::scrypto::abi::Type::Enum {
                            name: #ident_str.to_owned(),
                            variants: vec![
                                #(::scrypto::abi::Variant {
                                    name: #names.to_owned(),
                                    fields: #fields
                                }),*
                            ]
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Describe", &output);

    trace!("handle_describe() finishes");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_describe_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::abi::Describe for Test {
                    fn describe() -> ::scrypto::abi::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::scrypto::abi::Describe;

                        ::scrypto::abi::Type::Struct {
                            name: "Test".to_owned(),
                            fields: ::scrypto::abi::Fields::Named {
                                named: vec![("a".to_owned(), <u32>::describe())]
                            },
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn test_describe_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::abi::Describe for Test {
                    fn describe() -> ::scrypto::abi::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::scrypto::abi::Describe;

                        ::scrypto::abi::Type::Enum {
                            name: "Test".to_owned(),
                            variants: vec![
                                ::scrypto::abi::Variant {
                                    name: "A".to_owned(),
                                    fields: { ::scrypto::abi::Fields::Unit }
                                },
                                ::scrypto::abi::Variant {
                                    name: "B".to_owned(),
                                    fields: {
                                        ::scrypto::abi::Fields::Unnamed { unnamed: vec![<u32>::describe()] }
                                    }
                                },
                                ::scrypto::abi::Variant {
                                    name: "C".to_owned(),
                                    fields: {
                                        ::scrypto::abi::Fields::Named { named: vec![("x".to_owned(), <u8>::describe())] }
                                    }
                                }
                            ]
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn test_skip_field_1() {
        let input = TokenStream::from_str("struct Test {#[scrypto(skip)] a: u32}").unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::abi::Describe for Test {
                    fn describe() -> ::scrypto::abi::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::scrypto::abi::Describe;

                        ::scrypto::abi::Type::Struct {
                            name: "Test".to_owned(),
                            fields: ::scrypto::abi::Fields::Named { named: vec![] },
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn test_skip_field_2() {
        let input = TokenStream::from_str(
            "enum Test {A, B (#[scrypto(skip)] u32), C {#[scrypto(skip)] x: u8}}",
        )
        .unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::abi::Describe for Test {
                    fn describe() -> ::scrypto::abi::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::scrypto::abi::Describe;

                        ::scrypto::abi::Type::Enum {
                            name: "Test".to_owned(),
                            variants: vec![
                                ::scrypto::abi::Variant {
                                    name: "A".to_owned(),
                                    fields: { ::scrypto::abi::Fields::Unit }
                                },
                                ::scrypto::abi::Variant {
                                    name: "B".to_owned(),
                                    fields: {
                                        ::scrypto::abi::Fields::Unnamed { unnamed: vec![] }
                                    }
                                },
                                ::scrypto::abi::Variant {
                                    name: "C".to_owned(),
                                    fields: {
                                        ::scrypto::abi::Fields::Named { named: vec![] }
                                    }
                                }
                            ]
                        }
                    }
                }
            },
        );
    }
}
