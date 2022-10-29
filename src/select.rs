use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use std::iter;

#[derive(Debug)]
pub struct ColumnTarget {
    pub name: Ident,
    pub typename: Ident,
}

#[derive(Debug)]
pub enum Select {
    Spread(Ident),
    Anonymous(Vec<ColumnTarget>),
    Explicit { typename: Ident, fields: Vec<Ident> },
}

fn is_comma(tree: &TokenTree) -> bool {
    matches!(tree, TokenTree::Punct(p) if p.as_char() == ',')
}

pub fn parse(input: &mut Vec<TokenTree>) -> Result<Option<Select>, TokenStream> {
    if let Some(TokenTree::Ident(select)) = input.first() {
        if select == "SELECT" {
            if let Some(from_token_position) = input
                .iter()
                .position(|token| matches!(token, TokenTree::Ident(from) if from == "FROM"))
            {
                let (new_tokens, ret) = match &input[1..from_token_position] {
                    [TokenTree::Punct(p1), TokenTree::Punct(p2), TokenTree::Ident(typename)]
                        if p1.as_char() == '.' && p2.as_char() == '.' =>
                    {
                        (
                            vec![TokenTree::Punct(Punct::new('*', Spacing::Alone))],
                            Ok(Some(Select::Spread(typename.clone()))),
                        )
                    }

                    [TokenTree::Ident(typename), TokenTree::Group(mappings)]
                        if mappings.delimiter() == Delimiter::Brace =>
                    {
                        let tokens = mappings.stream().into_iter().collect::<Vec<_>>();
                        let (mut fields, mut columns) = (Vec::new(), Vec::new());
                        for column_def in tokens.split(is_comma) {
                            match column_def {
                                [TokenTree::Ident(field)] => {
                                    fields.push(field.clone());
                                    columns.push(column_def);
                                }
                                [TokenTree::Ident(field), TokenTree::Punct(p), col @ ..]
                                    if p.as_char() == ':' =>
                                {
                                    fields.push(field.clone());
                                    columns.push(col);
                                }
                                _ => {
                                    let column_def_text = format!("Explicit: {:?}", column_def);
                                    input.splice(1..from_token_position, iter::empty());
                                    return Err(quote!(compile_error!(#column_def_text);));
                                }
                            }
                        }

                        (
                            {
                                let mut columns_tokens = Vec::new();
                                columns.into_iter().for_each(|col| {
                                    columns_tokens.extend(col.iter().cloned());
                                    columns_tokens
                                        .push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                                });
                                columns_tokens.pop();
                                columns_tokens
                            },
                            Ok(Some(Select::Explicit {
                                typename: typename.clone(),
                                fields,
                            })),
                        )
                    }

                    _ => {
                        let (mut targets, mut columns) = (Vec::new(), Vec::new());
                        for column_def in input[1..from_token_position].split(is_comma) {
                            match column_def {
                                [TokenTree::Ident(name), TokenTree::Punct(p), TokenTree::Ident(typename)]
                                    if p.as_char() == ':' =>
                                {
                                    columns.push(&column_def[..1]);
                                    targets.push(ColumnTarget {
                                        name: name.clone(),
                                        typename: typename.clone(),
                                    });
                                }
                                [TokenTree::Ident(name), TokenTree::Punct(p), TokenTree::Ident(typename), TokenTree::Punct(at), def @ ..]
                                    if p.as_char() == ':' && at.as_char() == '@' =>
                                {
                                    columns.push(def);
                                    targets.push(ColumnTarget {
                                        name: name.clone(),
                                        typename: typename.clone(),
                                    });
                                }
                                _ => {
                                    let column_def_text = format!("Anon: {:?}", column_def);
                                    input.splice(1..from_token_position, iter::empty());
                                    return Err(quote!(compile_error!(#column_def_text);));
                                }
                            }
                        }

                        (
                            {
                                let mut columns_tokens = Vec::new();
                                columns.into_iter().for_each(|col| {
                                    columns_tokens.extend(col.iter().cloned());
                                    columns_tokens
                                        .push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                                });
                                columns_tokens.pop();
                                columns_tokens
                            },
                            Ok(Some(Select::Anonymous(targets))),
                        )
                    }
                };
                input.splice(1..from_token_position, new_tokens);
                ret
            } else {
                Err(quote!(compile_error!("missing FROM clause");))
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
