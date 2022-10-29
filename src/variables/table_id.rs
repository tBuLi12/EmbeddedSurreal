use proc_macro2::{Delimiter, Ident, Span, TokenTree};

use super::Variables;
use quote::quote;

impl Variables {
    pub fn replace_table_ids(&mut self, input: &mut Vec<TokenTree>) {
        let table_ids = input
            .windows(3)
            .enumerate()
            .filter_map(|(i, tokens)| match tokens {
                [TokenTree::Ident(table), TokenTree::Punct(colon), TokenTree::Group(id_expr)]
                    if colon.as_char() == ':' && id_expr.delimiter() == Delimiter::Brace =>
                {
                    let stream = id_expr.stream();
                    let fmt_expr = format!("{table}:{{}}");
                    Some((i, self.create(quote!(format!(#fmt_expr, #stream)))))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        for (i, var) in table_ids {
            input.splice(i..(i + 3), [TokenTree::Ident(var)]);
        }
    }
}

pub fn escape_table_ids(input: &mut [TokenTree]) {
    let colon_indexes = input
        .windows(3)
        .enumerate()
        .filter_map(|(i, tokens)| match tokens {
            [TokenTree::Ident(_), TokenTree::Punct(colon), TokenTree::Ident(_)]
                if colon.as_char() == ':' =>
            {
                Some(i + 1)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    for colon_idx in colon_indexes {
        input[colon_idx] = TokenTree::Ident(Ident::new("__rsql__colon", Span::call_site()))
    }
}
