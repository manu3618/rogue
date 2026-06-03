extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro::TokenTree::{Group, Ident, Punct};

#[proc_macro_derive(Category)]
pub fn derive_category(item: TokenStream) -> TokenStream {
    let name = item.clone().into_iter().nth(1).unwrap();
    let mut new_func = vec![
        format!("impl ::rogue_trait::EnumCategory for {name}{{"),
        "fn categories() -> Vec<Self>{".into(),
        "let mut v = Vec::new();".into(),
    ];

    let variants = match item.into_iter().nth(2).unwrap() {
        // should be a group delimited by curly braces
        Group(g) => g.stream().into_iter().collect::<Vec<_>>(),
        _ => unreachable!(),
    };
    let tokens = variants.windows(2);
    for elts in tokens {
        match (elts.first(), elts.get(1)) {
            (Some(Ident(x)), Some(Punct(_))) => new_func.push(format!("v.push(Self::{x});")),
            (Some(Ident(x)), Some(Group(_))) => {
                new_func.push(format!("v.push(Self::{x}(Default::default()));"))
            }
            _ => {
                // TODO: handle other cases
            }
        }
    }

    new_func.push("v}\n}".into());
    new_func.join("\n").parse().unwrap()
}
