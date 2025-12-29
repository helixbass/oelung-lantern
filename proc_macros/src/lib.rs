use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Path, Token,
};

#[proc_macro]
pub fn generate_sender(input: TokenStream) -> TokenStream {
    let spec: Spec = parse_macro_input!(input);

    quote! {
        #spec
    }
    .into()
}

struct Spec {
    pub enum_name: Ident,
    pub variant_name: Ident,
    pub type_: Path,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let variant_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let type_: Path = input.parse()?;
        Ok(Self {
            enum_name,
            variant_name,
            type_,
        })
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = format_ident!("{}Sender", self.variant_name);
        let type_ = &self.type_;
        let enum_name = &self.enum_name;
        let variant_name = &self.variant_name;

        quote! {
            #[derive(Clone)]
            struct #struct_name {
                pub sender: ::tokio::sync::mpsc::Sender<#enum_name>,
            }

            impl From<::tokio::sync::mpsc::Sender<#enum_name>> for #struct_name {
                fn from(value: ::tokio::sync::mpsc::Sender<#enum_name>) -> Self {
                    Self {
                        sender: value,
                    }
                }
            }

            #[::async_trait::async_trait]
            impl ::oelung_lantern::mpsc::Sender<#type_> for #struct_name {
                async fn send(&self, value: #type_) {
                    self.sender.send(#enum_name::#variant_name(value)).await.unwrap();
                }

                fn box_clone(&self) -> ::std::boxed::Box<dyn ::oelung_lantern::mpsc::Sender<#type_>> {
                    Box::new(self.clone())
                }
            }
        }
        .to_tokens(tokens)
    }
}
