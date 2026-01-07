use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use squalid::_d;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Expr, ExprClosure, Ident, Path, Token,
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
    pub expected_prefix: Expr,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<Ident> = _d();
        let mut state: Option<ExprClosure> = _d();
        let mut state_type: Option<Path> = _d();
        let mut send_and_receive: Option<Path> = _d();
        let mut expected_prefix: Option<Expr> = _d();

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
                "expected_prefix" => {
                    assert!(
                        expected_prefix.is_none(),
                        "Already saw 'expected_prefix' key"
                    );
                    expected_prefix = Some(input.parse()?);
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
            expected_prefix: expected_prefix.expect("Expected `expected_prefix`"),
        })
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let send_and_receive = &self.send_and_receive;
        let state_type = &self.state_type;
        let state = &self.state;
        let test_name = format_ident!("test_{}", self.name);
        let expected_prefix = &self.expected_prefix;

        quote! {
            #[tokio::test]
            async fn #test_name() -> Result<(), ::oelung::anyhow::Error> {
                let memory_backend = ::std::rc::Rc::new(::std::cell::RefCell::new(::oelung::BackendMemory::new(::oelung::Size {
                    height: 26,
                    width: 80,
                })));

                let mut renderer = ::oelung::RendererBuilder::default()
                    .backend(memory_backend.clone())
                    .build()?;

                let (sender, mut receiver) = ::tokio::sync::mpsc::channel::<World>(100);

                let expected_prefix = #expected_prefix;

                let (did_render_sender, did_render_receiver) = ::tokio::sync::mpsc::channel::<()>(100);

                ::tokio::spawn({
                    let sender = CrosstermSender::from(sender.clone());
                    let expected_prefix_len = expected_prefix.len();
                    async move {
                        use ::oelung_lantern::mpsc::Sender;
                        let mut num_renders = 0;
                        while let Some(()) = did_render_receiver.recv().await {
                            num_renders += 1;
                            if num_renders >= expected_prefix_len {
                                break;
                            }
                        }
                        sender
                            .send(::crossterm::event::Event::Key(::crossterm::event::KeyEvent {
                                code: ::crossterm::event::KeyCode::Char('q'),
                                modifiers: ::crossterm::event::KeyModifiers::NONE,
                                kind: ::crossterm::event::KeyEventKind::Press,
                                state: ::crossterm::event::KeyEventState::NONE,
                            }))
                            .await;
                    }
                });

                let state_callback = #state;
                let mut state = (state_callback)(Box::new(TestedSender::from(sender.clone())));

                render_screen(&mut renderer, &state)?;
                did_render_sender.send(()).await;

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
                            did_render_sender.send(()).await;
                        }
                        _ => {}
                    }
                    for effect in queued_effects {
                        ::tokio::spawn(effect);
                    }
                }

                ::oelung_lantern::assert_expected_screen_contents(&memory_backend.borrow(), expected_prefix);

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
        }
        .to_tokens(tokens)
    }
}
