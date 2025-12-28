use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Path,
};

#[proc_macro]
pub fn generate_sender(input: TokenStream) -> TokenStream {
    let spec: Spec = parse_macro_input!(input);

    quote! {{
        #spec
    }}
    .into()
}

struct Spec {
    pub enum_name: Ident,
    pub variant_name: Ident,
    pub type_: Path,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        unimplemented!()
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unimplemented!()
    }
}
