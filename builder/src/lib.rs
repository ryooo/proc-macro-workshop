use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(input as DeriveInput);
    let base_ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", base_ident);
    let mut fields_info = vec![];
    let mut types_info = vec![];
    match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    for f in fields.named.clone() {
                        fields_info.push(f.ident.unwrap());
                        types_info.push(f.ty);
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
            #(#fields_info: Option<#types_info>,)*
        }
        impl #base_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#fields_info: None,)*
                }
            }

        }

        impl #builder_ident {
            #(
                fn #fields_info(&mut self, #fields_info: #types_info) -> &mut Self {
                    self.#fields_info = Some(#fields_info);
                    self
                }
            )*
            pub fn build(&mut self) -> Result<#base_ident, Box<dyn error::Error>> {
                Ok(#base_ident {
                    #(#fields_info: self.#fields_info.clone().unwrap(),)*
                })
            }
        }
    };
    token.into()
}
