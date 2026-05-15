use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn avsc(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let relative_path = lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let full_path = std::path::Path::new(&manifest_dir).join(&relative_path);

    let content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(lit.span(), format!("failed to read {}: {e}", full_path.display()))
                .to_compile_error()
                .into();
        }
    };

    if let Err(e) = apache_avro::Schema::parse_str(&content) {
        return syn::Error::new(lit.span(), format!("invalid Avro schema in {}: {e}", full_path.display()))
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        {
            const _VALIDATED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #relative_path));
            _VALIDATED
        }
    };

    expanded.into()
}
