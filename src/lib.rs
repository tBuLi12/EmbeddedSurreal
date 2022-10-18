use std::{ops::Index, vec};

use proc_macro::{Delimiter, Ident, Punct, TokenStream, TokenTree};

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    TokenStream::new()
}

fn is_comma(tree: &TokenTree) -> bool {
    matches!(tree, TokenTree::Punct(p) if p.eq(&':'))
}

fn parse_select(input: &mut [TokenTree]) -> (&[TokenTree], Select) {
    let select_return = match input {
        // [TokenTree::Punct(p1), TokenTree::Punct(p2), TokenTree::Ident(typename), ..]
        //     if p1.eq(&'.') && p2.eq(&'.') =>
        // {
        //     Select {
        //         ret_handler: SelectReturn::Spread(typename.clone()),
        //         tables: vec![TokenTree::Punct(Punct::new(
        //             '*',
        //             proc_macro::Spacing::Alone,
        //         ))],
        //     }
        // }
        [TokenTree::Ident(typename), TokenTree::Group(mappings)]
            if mappings.delimiter() == Delimiter::Brace =>
        {
            enum Rest<'a> {
                One(Ident),
                Many(&'a [TokenTree]),
            }

            let (fields, columns) = mappings
                .stream()
                .into_iter()
                .collect::<Vec<TokenTree>>()
                .split(is_comma)
                .map(|column_def| match column_def {
                    [TokenTree::Ident(field), TokenTree::Punct(p), col @ ..] if p.eq(&':') => {
                        (field.clone(), Rest::Many(col))
                    }
                    [TokenTree::Ident(field)] => (field.clone(), Rest::One(field.clone())),
                })
                .unzip::<Ident, Rest, Vec<_>, Vec<_>>();

            Select {
                ret_handler: SelectReturn::Explicit(typename.clone(), fields),
                columns: columns
                    .into_iter()
                    .map(|column| match column {
                        Rest::One(col_name) => vec![TokenTree::Ident(col_name)].into_iter(),
                        Rest::Many(expression) => expression.to_vec().into_iter(),
                    })
                    .intersperse(TokenTree::Punct(Punct::new(',', Spacing::Alone)))
                    .flatten()
                    .collect(),
            }
        }

        _ => SelectReturn::Anonymous(
            input
                .split(is_comma)
                .map(|column_def| match column_def {
                    [.., TokenTree::Punct(p), TokenTree::Ident(typename)] if p.eq(&':') => {
                        typename.clone()
                    }
                })
                .collect(),
        ),
    };
    return (&[], select_return);
}

struct Select {
    ret_handler: SelectReturn,
    columns: Vec<TokenTree>,
}

enum SelectReturn {
    Spread(Ident),
    Explicit(Ident, Vec<Ident>),
    Anonymous(Vec<Ident>),
}
