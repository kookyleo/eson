use std::fs;

// build0
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, FnArg, ItemFn, Meta, PatType, Signature, Type, TypePath, visit::Visit};

struct EsonVisitor {
    pub methods: Vec<(Ident, Signature)>,
}

fn has_udf_attribute(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Meta::Path(path) = &attr.meta {
            for segment in &path.segments {
                if segment.ident == "udf" {
                    return true;
                }
            }
        }
    }
    false
}

impl<'ast> Visit<'ast> for EsonVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        let attrs = &i.attrs;
        if has_udf_attribute(attrs) {
            self.methods.push((i.sig.ident.clone(), i.sig.clone()));
        }
    }
}

fn main() {
    let src = fs::read_to_string("src/bin/executor.rs").expect("Unable to read file");
    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    let mut visitor = EsonVisitor {
        methods: Vec::new(),
    };

    visitor.visit_file(&syntax);

    let mut tokens: Vec<TokenStream> = Vec::new();
    for (ident, sig) in visitor.methods {
        let variant_name = ident;
        let args = sig.inputs;

        // 遍历读取参数类型
        let mut arg_types: Vec<TokenStream> = Vec::new();
        for arg in args {
            match arg {
                FnArg::Typed(PatType { ty, .. }) => match ty.as_ref() {
                    Type::Path(TypePath { path, .. }) => {
                        let segments = &path.segments.clone().into_iter().collect::<Vec<_>>();
                        arg_types.push(quote! {
                            #(#segments)::*
                        });
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // 将 arg_types 转化为 tokens, 用于 quote!
        tokens.push(quote! {
            #variant_name(#(#arg_types),*)
        });
    }

    let udf_calls_enum = quote! {
        use crate::{JsonInt, JsonFloat, JsonString, JsonNull, JsonBool, JsonArray, JsonObject};

        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub enum UdfCall {
            #(#tokens),*
        }
    };

    // 输出到文件
    std::fs::write("src/udf_calls.rs", udf_calls_enum.to_string()).expect("Unable to write file");
    std::process::Command::new("rustfmt")
        .arg("src/udf_calls.rs")
        .status()
        .expect("Unable to run rustfmt");
}
