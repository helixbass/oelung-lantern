use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use squalid::_d;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, ExprClosure, Ident, Path, Token,
};

pub fn render_multiple_test(input: TokenStream) -> TokenStream {
    let spec: Spec = parse_macro_input!(input);

    quote! {
        #spec
    }
    .into()
}

struct Spec {
    pub name: Ident,
    pub state: ExprClosure,
    pub state_type: Path,
    pub send_and_receive: Path,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<Ident> = _d();
        let mut state: Option<ExprClosure> = _d();
        let mut state_type: Option<Path> = _d();
        let mut send_and_receive: Option<Path> = _d();
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
                "state_type" => {
                    assert!(state_type.is_none(), "Already saw 'state_type' key");
                    state_type = Some(input.parse()?);
                }
                "send_and_receive" => {
                    assert!(
                        send_and_receive.is_none(),
                        "Already saw 'send_and_receive' key"
                    );
                    send_and_receive = Some(input.parse()?);
                }
                key => return Err(input.error(format!("Unexpected key `{key}`"))),
            }
            input.parse::<Option<Token![,]>>()?;
        }

        Ok(Self {
            name: name.expect("Expected `name`"),
            state: state.expect("Expected `state`"),
            state_type: state_type.expect("Expected `state_type`"),
            send_and_receive: send_and_receive.expect("Expected `send_and_receive`"),
        })
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let send_and_receive = &self.send_and_receive;
        let state_type = &self.state_type;
        let state = &self.state;
        let test_name = format_ident!("test_{}", self.name);

        quote! {
            #[tokio::test]
            async fn #test_name() -> Result<(), ::oelung::anyhow::Error> {
                let mut renderer = ::oelung::RendererBuilder::default().build()?;

                let (sender, mut receiver) = ::tokio::sync::mpsc::channel::<World>(100);

                listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

                let state_callback = #state;
                let mut state = (state_callback)(Box::new(TestedSender::from(sender.clone())));

                render_screen(&mut renderer, &state)?;

                while let Some(world) = receiver.recv().await {
                    let mut queued_effects: Vec<::std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
                    match world {
                        World::Crossterm(::crossterm::event::Event::Key(key)) if key.code == ::crossterm::event::KeyCode::Char('q') => {
                            break;
                        }
                        World::Tested(event) => {
                            use ::oelung_lantern::ReceiveEvent;
                            state.receive(&event, |future| queued_effects.push(future))?;
                            render_screen(&mut renderer, &state)?;
                        }
                        _ => {}
                    }
                    for effect in queued_effects {
                        ::tokio::spawn(effect);
                    }
                }

                Ok(())
            }

            fn render_screen(renderer: &mut ::oelung::Renderer, state: &#state_type) -> Result<(), anyhow::Error> {
                renderer.render(::oelung::soft! {
                    %state
                })?;

                Ok(())
            }

            enum World {
                Crossterm(::crossterm::event::Event),
                Tested(#send_and_receive),
            }

            ::oelung_lantern::generate_sender!(World, Crossterm, ::crossterm::event::Event);
            ::oelung_lantern::generate_sender!(World, Tested, #send_and_receive);

            fn listen_to_crossterm_events(sender: CrosstermSender) {
                ::tokio::spawn(async move {
                    use ::tokio_stream::StreamExt;
                    use ::oelung_lantern::mpsc::Sender;
                    let mut event_stream = ::crossterm::event::EventStream::new();

                    while let Some(Ok(event)) = event_stream.next().await {
                        sender.send(event).await;
                    }

                    panic!("kill everything")
                });
            }
        }
        .to_tokens(tokens)
    }
}
