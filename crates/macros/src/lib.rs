use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn exf(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
