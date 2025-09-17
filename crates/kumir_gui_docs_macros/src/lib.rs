extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::{
    error::Error,
    fs::{self, read_dir},
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
    _comma3: Token![,],
    selected: Expr,
}

impl Parse for DocsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DocsInput {
            ui: input.parse()?,
            _comma1: input.parse()?,
            cache: input.parse()?,
            _comma2: input.parse()?,
            path: input.parse()?,
            _comma3: input.parse()?,
            selected: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn docs(input: TokenStream) -> TokenStream {
    let DocsInput {
        ui,
        cache,
        path,
        selected,
        ..
    } = parse_macro_input!(input as DocsInput);

    let path_string = path.value();
    let path = PathBuf::from(&path_string);

    let md_file_paths = expand_dir(&path)
        .unwrap_or_else(|e| panic!("Failed to scan directory '{}': {}", path.display(), e));

    if md_file_paths.is_empty() {
        panic!("No .md files found in directory '{}'", path.display());
    }

    let file_titles = get_file_titles(&md_file_paths)
        .unwrap_or_else(|e| panic!("Failed to get file title '{}': {}", path.display(), e));

    let expanded = quote! {
        {
            use std::collections::HashMap;
            let mut selected_names_map: HashMap<String, String> = HashMap::new();
            #(
                selected_names_map.insert(#md_file_paths.to_string(), #file_titles.to_string());
            )*
            egui::ComboBox::from_id_salt("mode")
                .selected_text(selected_names_map.get(&**#selected).unwrap_or(&"No value selected".to_string()))
                .show_ui(#ui, |ui| {
                    #(

                        let full_path = #md_file_paths.to_owned();

                        ui.selectable_value(
                            #selected,
                            full_path,
                            #file_titles,
                        );
                    )*
                });

            #(
                if *#selected == #md_file_paths {
                    egui_commonmark::commonmark_str!(#ui, #cache, #md_file_paths);
                }
            )*
        }
    };

    expanded.into()
}

fn get_file_titles(paths: &Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
    paths
        .iter()
        .map(|path| {
            let file: String = fs::read_to_string(path)?;
            let title = file
                .lines()
                .next()
                .ok_or(format!("Error file is empty"))?
                .replace('#', "");
            Ok(title)
        })
        .collect()
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
