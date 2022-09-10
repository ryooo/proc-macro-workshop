use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type, PathArguments, GenericArgument};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(input as DeriveInput);
    let base_ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", base_ident);
    // dbg!(&input.data);

    let mut fields_token = vec![];
    let mut types_token = vec![];
    let mut required_fields_token = vec![];
    let mut required_types_token = vec![];
    let mut required_field_names_token = vec![];
    let mut option_fields_token = vec![];
    let mut option_inner_types_token = vec![];
    match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    for f in fields.named.clone() {
                        match &f.ty {
                            Type::Path(path) => {
                                match path.path.segments.first() {
                                    Some(segment) => {
                                        if segment.ident == "Option" {
                                            option_fields_token.push(f.ident.clone().unwrap());

                                            match &segment.arguments {
                                                PathArguments::AngleBracketed(inner) => {
                                                    match &inner.args.first().unwrap() {
                                                        GenericArgument::Type(inner_type) => {
                                                            option_inner_types_token.push(inner_type.clone())
                                                        }
                                                        _ => panic!("unknown type")
                                                    }
                                                }
                                                _ => panic!("unknown type")
                                            }
                                        } else {
                                            required_fields_token.push(f.ident.clone().unwrap());
                                            required_field_names_token.push(f.ident.clone().unwrap().to_string());
                                            required_types_token.push(f.ty.clone())
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                        fields_token.push(f.ident.clone().unwrap());
                        types_token.push(f.ty);
                    };
                }
                _ => panic!("fields should be Named")
            }
        }
        _ => {
            panic!("data should be Struct")
        }
    }

    let token = quote! {
        pub struct #builder_ident {
            #(#required_fields_token: Option<#required_types_token>,)*
            #(#option_fields_token: Option<#option_inner_types_token>,)*
        }
        impl #base_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#fields_token: None,)*
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
            pub fn build(&mut self) -> Result<#base_ident, Box<dyn std::error::Error>> {
                #(
                    if self.#required_fields_token.is_none() {
                        return Err(format!("{} is required.", #required_field_names_token).into())
                    }
                )*
                Ok(#base_ident {
                    #(#required_fields_token: self.#required_fields_token.clone().unwrap(),)*
                    #(#option_fields_token: self.#option_fields_token.clone(),)*
                })
            }
        }
    };
    token.into()
}
