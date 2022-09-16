use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type, PathArguments, GenericArgument, NestedMeta, Meta, Lit};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(input as DeriveInput);
    let base_ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", base_ident);
    // dbg!(&input.data);

    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => fields,
                _ => panic!("fields should be Named")
            }
        }
        _ => panic!("data should be Struct")
    };

    // 変数を収集
    let mut required_fields_token = vec![];
    let mut required_types_token = vec![];
    let mut required_field_names_token = vec![];
    let mut option_fields_token = vec![];
    let mut option_inner_types_token = vec![];
    let mut each_fields_token = vec![];
    let mut each_types_token = vec![];
    let mut each_inner_types_token = vec![];
    let mut each_method_names_token = vec![];
    for f in fields.named.clone() {
        let mut has_each_attribute = false;
        for attr in f.attrs.clone() {
            let attr_meta = attr.parse_meta().unwrap();
            if is_target_path_ident(&attr_meta.path(), "builder") {
                match attr_meta {
                    Meta::List(meta_list) => {
                        match meta_list.nested.first().unwrap() {
                            NestedMeta::Meta(meta) => {
                                if is_target_path_ident(&meta.path(), "each") {
                                    match meta {
                                        Meta::NameValue(name_value) => {
                                            match name_value.clone().lit {
                                                Lit::Str(lit_str) => {
                                                    each_fields_token.push(f.ident.clone().unwrap());
                                                    each_types_token.push(f.ty.clone());
                                                    each_inner_types_token.push(detect_option_inner_type(&f.ty));
                                                    each_method_names_token.push(format_ident!("{}", lit_str.value()));
                                                    has_each_attribute = true
                                                },
                                                _ => panic!("wrong builder attribute usage."),
                                            }
                                        },
                                        _ => panic!("wrong builder attribute usage."),
                                    }
                                }
                            },
                            _ => panic!("wrong builder attribute usage."),
                        }
                    },
                    _ => panic!("wrong builder attribute usage.")
                }
            }
        }
        if !has_each_attribute {
            if is_option(&f.ty) {
                option_fields_token.push(f.ident.clone().unwrap());
                option_inner_types_token.push(detect_option_inner_type(&f.ty));
            } else {
                required_fields_token.push(f.ident.clone().unwrap());
                required_field_names_token.push(f.ident.clone().unwrap().to_string());
                required_types_token.push(f.ty.clone())
            }
        }
    };

    let token = quote! {
        pub struct #builder_ident {
            #(#required_fields_token: Option<#required_types_token>,)*
            #(#option_fields_token: Option<#option_inner_types_token>,)*
            #(#each_fields_token: #each_types_token,)*
        }
        impl #base_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#required_fields_token: None,)*
                    #(#option_fields_token: None,)*
                    #(#each_fields_token: vec![],)*
                }
            }
        }

        impl #builder_ident {
            #(
                fn #required_fields_token(&mut self, #required_fields_token: #required_types_token) -> &mut Self {
                    self.#required_fields_token = Some(#required_fields_token);
                    self
                }
            )*
            #(
                fn #option_fields_token(&mut self, #option_fields_token: #option_inner_types_token) -> &mut Self {
                    self.#option_fields_token = Some(#option_fields_token);
                    self
                }
            )*
            #(
                fn #each_method_names_token(&mut self, val: #each_inner_types_token) -> &mut Self {
                    self.#each_fields_token.push(val);
                    self
                }
            )*
            pub fn build(&mut self) -> Result<#base_ident, Box<dyn std::error::Error>> {
                #(
                    if self.#required_fields_token.is_none() {
                        return Err(format!("{} is required.", #required_field_names_token).into())
                    }
                )*
                Ok(#base_ident {
                    #(#required_fields_token: self.#required_fields_token.clone().unwrap(),)*
                    #(#option_fields_token: self.#option_fields_token.clone(),)*
                    #(#each_fields_token: self.#each_fields_token.clone(),)*
                })
            }
        }
    };
    token.into()
}

fn is_target_path_ident(path: &syn::Path, target: &str) -> bool {
    return match path.segments.first() {
        Some(segment) => segment.ident == target,
        _ => false,
    };
}

fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            match path.path.segments.first() {
                Some(segment) => segment.ident == "Option",
                _ => false,
            }
        }
        _ => false,
    }
}

fn detect_option_inner_type(ty: &Type) -> Type {
    let segment = match ty {
        Type::Path(path) => {
            match path.path.segments.first() {
                Some(segment) => segment,
                _ => panic!("not option."),
            }
        }
        _ => panic!("not option."),
    };
    match &segment.arguments {
        PathArguments::AngleBracketed(inner) => {
            match &inner.args.first().unwrap() {
                GenericArgument::Type(inner_type) => inner_type.clone(),
                _ => panic!("unknown type")
            }
        }
        _ => panic!("unknown type")
    }
}