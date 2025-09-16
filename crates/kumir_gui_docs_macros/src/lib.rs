extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::{
    error::Error,
    fs::read_dir,
    path::{Path, PathBuf},
};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token, parse_macro_input};

struct DocsInput {
    ui: Expr,
    _comma1: Token![,],
    cache: Expr,
    _comma2: Token![,],
    path: LitStr,
}

impl Parse for DocsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DocsInput {
            ui: input.parse()?,
            _comma1: input.parse()?,
            cache: input.parse()?,
            _comma2: input.parse()?,
            path: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn docs(input: TokenStream) -> TokenStream {
    let DocsInput {
        ui, cache, path, ..
    } = parse_macro_input!(input as DocsInput);

    let path_string = path.value();
    let path = PathBuf::from(&path_string);

    let md_file_paths = expand_dir(&path)
        .unwrap_or_else(|e| panic!("Failed to scan directory '{}': {}", path.display(), e));

    if md_file_paths.is_empty() {
        panic!("No .md files found in directory '{}'", path.display());
    }

    let expanded = quote! {
        {
            #( egui_commonmark::commonmark_str!(#ui, #cache, #md_file_paths); )*
        }
    };

    expanded.into()
}

fn expand_dir(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut md_files = Vec::new();
    let children = read_dir(path)?;

    for child_result in children {
        let child = child_result?;
        let child_path = child.path();
        let file_type = child.file_type()?;

        if file_type.is_dir() {
            let sub_files = expand_dir(&child_path)?;
            md_files.extend(sub_files);
        } else if file_type.is_file() {
            if child_path.extension().map_or(false, |ext| ext == "md") {
                let path_str = child_path
                    .to_str()
                    .ok_or("Path contains invalid UTF-8")?
                    .to_string()
                    .replace('\\', "/");
                md_files.push(path_str);
            }
        }
    }

    Ok(md_files)
}
