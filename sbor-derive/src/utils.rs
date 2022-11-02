use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Attribute;
use syn::Generics;
use syn::Path;
use syn::TypeGenerics;
use syn::WhereClause;

#[allow(dead_code)]
pub fn print_generated_code<S: ToString>(kind: &str, code: S) {
    if let Ok(mut proc) = Command::new("rustfmt")
        .arg("--emit=stdout")
        .arg("--edition=2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(code.to_string().as_bytes()).unwrap();
        }
        if let Ok(output) = proc.wait_with_output() {
            if output.status.success() {
                println!(
                    "{}\n{}\n{}\n{}\n",
                    "-".repeat(kind.len()),
                    kind,
                    "-".repeat(kind.len()),
                    String::from_utf8(output.stdout).unwrap()
                );
            }
        }
    }
}

pub fn custom_type_id(attrs: &Vec<Attribute>) -> Option<Path> {
    for attr in attrs {
        if attr.path.is_ident("custom_type_id") {
            if let Ok(parsed) = attr.parse_args::<Path>() {
                return Some(parsed);
            }
        }
    }
    None
}

pub fn is_skipped(f: &syn::Field, id: &str) -> bool {
    f.attrs.iter().any(|attr| {
        if attr.path.is_ident("skip") {
            if let Ok(parsed) = attr.parse_args_with(Punctuated::<Path, Comma>::parse_terminated) {
                if parsed.iter().any(|x| x.is_ident(id)) {
                    return true;
                }
            }
        }
        return false;
    })
}

pub fn is_decode_skipped(f: &syn::Field) -> bool {
    is_skipped(f, "Decode")
}

pub fn is_encode_skipped(f: &syn::Field) -> bool {
    is_skipped(f, "Encode")
}

pub fn build_generics(
    generics: &Generics,
    custom_type_id: Option<Path>,
) -> syn::Result<(Generics, TypeGenerics, Option<&WhereClause>, Path)> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Unwrap for mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let sbor_cti = if let Some(path) = custom_type_id {
        path
    } else {
        // Note that this above logic requires no use of CTI generic param by the input type.
        // TODO: better to report error OR take an alternative name if already exists
        impl_generics
            .params
            .push(parse_quote!(CTI: ::sbor::type_id::CustomTypeId));
        parse_quote! { CTI }
    };

    Ok((impl_generics, ty_generics, where_clause, sbor_cti))
}
