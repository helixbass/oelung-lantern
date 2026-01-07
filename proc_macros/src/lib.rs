use proc_macro::TokenStream;

mod generate_sender;
mod render_multiple_test;

#[proc_macro]
pub fn generate_sender(input: TokenStream) -> TokenStream {
    generate_sender::generate_sender(input)
}

#[proc_macro]
pub fn render_multiple_test(input: TokenStream) -> TokenStream {
    render_multiple_test::render_multiple_test(input)
}
