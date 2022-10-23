use proc_macro::{Delimiter, Ident, TokenStream, TokenTree};
pub struct Select {
    columns: Vec<TokenTree>,
    return_parser: ReturnParser,
}

struct ColumnTarget {
    name: Ident,
    typename: Ident,
}

enum ReturnParser {
    Spread(Ident),
    Anonymous(Vec<Option<ColumnTarget>>),
    Explicit {
        typename: Ident,
        fields: Vec<Option<Ident>>,
    },
}

fn is_comma(tree: &TokenTree) -> bool {
    matches!(tree, TokenTree::Punct(p) if p.eq(&':'))
}

pub fn parse_select(input: &mut Vec<TokenTree>) -> Option<Select> {
    if let Some(TokenTree::Ident(select)) = input.first() {
        if select.to_string() != "SELECT" {}
    }

    match &input[..] {
        [TokenTree::Punct(p1), TokenTree::Punct(p2), TokenTree::Ident(typename)]
            if p1.as_char() == '.' && p2.as_char() == '.' =>
        {
            Select {
                columns: " * ".into(),
                return_parser: ReturnParser::Spread(typename.clone()),
            }
        }

        [TokenTree::Ident(typename), TokenTree::Group(mappings)]
            if mappings.delimiter() == Delimiter::Brace =>
        {
            let tokens = mappings.stream().into_iter().collect::<Vec<TokenTree>>();

            let (fields, columns) = tokens
                .split(is_comma)
                .map(|column_def| match column_def {
                    [TokenTree::Ident(field)] => (Some(field.clone()), &column_def[..1]),
                    [TokenTree::Ident(field), TokenTree::Punct(p), col @ ..] if p.eq(&':') => {
                        (Some(field.clone()), col)
                    }
                    _ => (None, &[][..]),
                })
                .unzip::<Option<Ident>, &[TokenTree], Vec<_>, Vec<_>>();

            Select {
                columns: {
                    let mut columns_str = String::from(" ");
                    columns.into_iter().for_each(|col| {
                        columns_str +=
                            &(TokenStream::from_iter(col.into_iter().cloned()).to_string() + " ,");
                    });
                    columns_str.pop();
                    columns_str
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
                .map(|column_def| match column_def {
                    [TokenTree::Ident(name), TokenTree::Punct(p), TokenTree::Ident(typename)]
                    if p.as_char() == ':' =>
                    {
                        (Some(ColumnTarget { name: name.clone(), typename: typename.clone() }), &column_def[..1])
                    }
                    [TokenTree::Ident(name), TokenTree::Punct(p), TokenTree::Ident(typename), TokenTree::Punct(at), def @ ..] if p.eq(&':') && at.eq(&'@') => {
                        (Some(ColumnTarget{ name: name.clone(), typename: typename.clone() }), def)
                    }
                    _ => (None, &[][..])
                })
                .unzip::<Option<ColumnTarget>, &[TokenTree], Vec<_>, Vec<_>>();
            Select {
                columns: columns
                    .into_iter()
                    .flatten()
                    .cloned()
                    .collect::<TokenStream>()
                    .to_string(),
                return_parser: ReturnParser::Anonymous(targets),
            }
        }
    }
}
