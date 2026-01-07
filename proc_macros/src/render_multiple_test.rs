use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use squalid::_d;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Expr, Ident, Path, Token,
};

render_multiple_test! {
    name => snake_spinner
    state => SnakeSpinner::new(
        None,
        Some(Color::Red),
        SENDER
    )
    receive => snake::Tick
    send => snake::Tick
}
pub fn render_multiple_test(input: TokenStream) -> TokenStream {
    let spec: Spec = parse_macro_input!(input);

    quote! {
        #spec
    }
    .into()
}

struct Spec {
    pub name: Ident,
    pub state: Expr,
    pub receive: Path,
    pub send: Path,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<Ident> = _d();
        let mut state: Option<Expr> = _d();
        let mut receive: Option<Path> = _d();
        let mut send: Option<Path> = _d();
        // while input.peek(Ident) {
        while !input.is_empty() {
            let key = input.parse::<Ident>().unwrap().to_string();
            input.parse::<Token![=>]>()?;
            match &*key {
                "name" => {
                    assert!(name.is_none(), "Already saw 'name' key");
                    name = Some(input.parse()?);
                }
                "state" => {
                    assert!(state.is_none(), "Already saw 'state' key");
                    state = Some(input.parse()?);
                }
                "receive" => {
                    assert!(receive.is_none(), "Already saw 'receive' key");
                    receive = Some(input.parse()?);
                }
                "send" => {
                    assert!(send.is_none(), "Already saw 'send' key");
                    send = Some(input.parse()?);
                }
                key => return Err(input.error(format!("Unexpected key `{key}`"))),
            }
            input.parse::<Option<Token![,]>>()?;
        }
        Ok(Self {
            name: name.expect("Expected `name`"),
            state: state.expect("Expected `state`"),
            receive: receive.expect("Expected `receive`"),
            send: send.expect("Expected `send`"),
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
