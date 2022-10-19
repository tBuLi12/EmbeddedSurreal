use proc_macro::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    TokenStream::new()
}

fn is_comma(tree: &TokenTree) -> bool {
    matches!(tree, TokenTree::Punct(p) if p.eq(&':'))
}

fn parse_select(input: Box<[TokenTree]>) -> Select {
    match &*input {
        [TokenTree::Punct(p1), TokenTree::Punct(p2), TokenTree::Ident(typename)]
            if p1.eq(&'.') && p2.eq(&'.') =>
        {
            Select {
                columns: vec![TokenTree::Punct(Punct::new('*', Spacing::Alone))],
                return_parser: ReturnParser::Spread(typename.clone()),
            }
        }

        [TokenTree::Ident(typename), TokenTree::Group(mappings)]
            if mappings.delimiter() == Delimiter::Brace =>
        {
            enum Rest<'a> {
                One(Ident),
                Many(&'a [TokenTree]),
            }

            let tokens = mappings.stream().into_iter().collect::<Vec<TokenTree>>();

            let (fields, columns) = tokens
                .split(is_comma)
                .map(|column_def| match column_def {
                    [TokenTree::Ident(field)] => (field.clone(), &column_def[..1]),
                    [TokenTree::Ident(field), TokenTree::Punct(p), col @ ..] if p.eq(&':') => {
                        (field.clone(), col)
                    }
                })
                .unzip::<Ident, &[TokenTree], Vec<_>, Vec<_>>();

            Select {
                columns: {
                    let mut columns_tokens = Vec::<TokenTree>::with_capacity(columns.len() * 2 - 1);
                    columns.into_iter().for_each(|col| {
                        columns_tokens.extend(col.into_iter().cloned());
                        columns_tokens.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)))
                    });
                    columns_tokens.pop();
                    columns_tokens
                },
                return_parser: ReturnParser::Explicit {
                    typename: typename.clone(),
                    fields,
                },
            }
        }

        _ => {
            let (targets, columns) = input
                .split(is_comma)
                .map(|column_def| match *column_def {
                    [TokenTree::Ident(name), TokenTree::Punct(p), TokenTree::Ident(typename)]
                    if p.eq(&':') =>
                    {
                        (ColumnTarget { name , typename }, &column_def[..1])
                    }
                    [TokenTree::Ident(name), TokenTree::Punct(p), TokenTree::Ident(typename), TokenTree::Punct(at), ref def @ ..] if p.eq(&':') && at.eq(&'@') => {
                        (ColumnTarget { name, typename }, def)
                    }
                })
                .unzip::<ColumnTarget, &[TokenTree], Vec<_>, Vec<_>>();
            Select {
                columns: columns.into_iter().flatten().cloned().collect(),
                return_parser: ReturnParser::Anonymous(targets),
            }
        }
    }
}

struct Select {
    columns: Vec<TokenTree>,
    return_parser: ReturnParser,
}

struct ColumnTarget {
    name: Ident,
    typename: Ident,
}

enum ReturnParser {
    Spread(Ident),
    Explicit { typename: Ident, fields: Vec<Ident> },
    Anonymous(Vec<ColumnTarget>),
}

fn dupa() {
    let mut a: Vec<Vec<usize>> = vec![vec![], vec![], vec![]];

    match &mut *a {
        [v, ..] => v[0],
        [] => 1,
        _ => 1,
    };
}
