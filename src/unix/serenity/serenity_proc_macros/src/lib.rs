#![feature(proc_macro_span)]
extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use std::fs::read_to_string;
use std::path::Path;

#[proc_macro]
pub fn generate_errno_definitions(input: TokenStream) -> TokenStream {
    let mut result = String::from("errno_definitions! {");
    #[derive(Debug)]
    enum State {
        Skipping,
        SeenHash,
        SeenDefine,
        SeenMacroName,
        Reading,
    }
    let mut state = State::Skipping;

    let mut it = input.into_iter();
    let path_item = it.next().unwrap();
    let errno_file_contents = if let TokenTree::Literal(lit) = path_item {
        assert!(it.next().is_none());
        let serenity_root = Path::new(env!("SERENITY_ROOT"));
        let path_fragment_string = lit.span().source_text().unwrap();
        let path_fragment = Path::new(&path_fragment_string[1..path_fragment_string.len() - 1]);
        let path = serenity_root.join(path_fragment);
        read_to_string(path).unwrap().replace("\\\n", "")
    } else {
        panic!("Expected a literal path");
    };

    let mut index = 0;
    for token in errno_file_contents.parse::<TokenStream>().unwrap() {
        match state {
            State::Skipping => {
                if let TokenTree::Punct(punct) = token {
                    if punct.as_char() == '#' {
                        state = State::SeenHash;
                    }
                }
            }
            State::SeenHash => {
                if let TokenTree::Ident(ident) = token {
                    if ident.to_string() == "define" {
                        state = State::SeenDefine;
                        continue;
                    }
                }
                state = State::Skipping;
            }
            State::SeenDefine => {
                if let TokenTree::Ident(ident) = token {
                    if ident.to_string() == "ENUMERATE_ERRNO_CODES" {
                        state = State::SeenMacroName;
                        continue;
                    }
                }
                state = State::Skipping;
            }
            State::SeenMacroName => {
                if let TokenTree::Group(group) = token {
                    state = State::Reading;
                    continue;
                }
            }
            State::Reading => {
                if let TokenTree::Ident(ident) = &token {
                    if ident.to_string() == "enum" {
                        break;
                    }
                } else if let TokenTree::Group(_) = &token {
                    result.push_str(format!("{},", index).as_str());
                    result.push_str(&token.to_string());
                    index += 1;
                    continue;
                }
            }
        }
    }

    result.push('}');
    result.parse().unwrap()
}
