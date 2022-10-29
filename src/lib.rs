use proc_macro::TokenStream;
use quote::quote;

mod select;
mod variables;
use variables::table_id;

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let mut tokens = proc_macro2::TokenStream::from(input)
        .into_iter()
        .collect::<Vec<_>>();

    let result_parser = match select::parse(&mut tokens) {
        Err(err) => return err.into(),
        Ok(parser) => parser,
    };

    let mut vars = variables::Variables::new();

    vars.replace_table_ids(&mut tokens);
    vars.replace_variables(&mut tokens);

    table_id::escape_table_ids(&mut tokens);

    let query_text = proc_macro2::TokenStream::from_iter(tokens)
        .to_string()
        .replace("__rsql__var", "$v")
        .replace(" __rsql__colon ", ":");

    let var_tokens = vars.get_tokens();
    let query_call_tokens = quote!(db.query(#query_text, #var_tokens));

    match result_parser {
        Some(select::Select::Spread(typename)) => {
            quote!(#query_call_tokens.map(serde_json::from_value::<Vec::<#typename>>))
        }
        Some(select::Select::Anonymous(columns)) => {
            let names = columns.iter().map(|col| col.name);
            let types = columns.iter().map(|col| col.typename);
            quote!({
                #[derive(serde::Deserialize)]
                struct QueryReturn {
                    #(#names: #types),*
                }
                #query_call_tokens.map(serde_json::from_value::<Vec::<QueryReturn>>)
            })
        }
        Some(select::Select::Explicit { fields, typename }) => {
            quote!(#query_call_tokens.map(serde_json::from_value::<Vec::<QueryReturn>>))
        }
        None => query_call_tokens,
    }
    .into()

    // quote!(db.query(#query_text, #var_tokens)).into()
    // let toks = quote!(db.query(#query_text, #var_tokens)).to_string();
    // quote!(println!("{}", #toks)).into()
}
