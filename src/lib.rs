use proc_macro::{Delimiter, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
mod select;

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let tokens = input.into_iter().collect::<Vec<_>>();
    match &tokens[..] {
        [TokenTree::Ident(select), rest @ ..] if select.to_string() == "SELECT" => {
            let mut rest = rest
                .split(|tree| matches!(tree, TokenTree::Ident(from) if from.to_string() == "FROM"));
            match (rest.next(), rest.next()) {
                (Some(columns), Some(tables)) => {
                    parse_select(columns);
                    let mut vars = Variables::new();
                }
                _ => (),
            }
        }
        _ => (),
    };
    TokenStream::new()
}

struct Variables {
    values: Vec<TokenStream>,
    last_id: usize,
}

impl Variables {
    fn new() -> Variables {
        Variables {
            values: Vec::new(),
            last_id: 0,
        }
    }

    // fn create(&mut self, value: &[TokenTree]) -> Ident {
    //     self.values.push(value);
    //     self.last_id += 1;
    //     Ident::new(&format!("v{}", self.last_id), Span::call_site())
    // }

    // fn replace_table_ids(&mut self, query: &mut [TokenTree]) -> impl Iterator<Item = TokenTree> {
    //     query
    //         .windows(3)
    //         .enumerate()
    //         .filter_map(|(i, tokens)| match tokens {
    //             [TokenTree::Ident(table), TokenTree::Punct(colon), TokenTree::Group(id_expr)]
    //                 if colon.as_char() == ':' =>
    //             {
    //                 Some((i, self.create(i)))
    //             }
    //             _ => None,
    //         });
    //     ()
    // }
}
