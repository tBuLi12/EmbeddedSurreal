use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};

pub mod table_id;

#[derive(Debug)]
pub struct Variables {
    values: Vec<TokenStream>,
    last_id: usize,
}

impl Variables {
    pub fn new() -> Variables {
        Variables {
            values: Vec::new(),
            last_id: 0,
        }
    }

    pub fn create(&mut self, value: TokenStream) -> Ident {
        self.values.push(value);
        self.last_id += 1;
        Ident::new(&format!("__rsql__var{}", self.last_id), Span::call_site())
    }

    pub fn replace_variables(&mut self, input: &mut [TokenTree]) {
        for token in input {
            if let TokenTree::Group(var_expr) = token {
                if var_expr.delimiter() == Delimiter::Brace {
                    *token = TokenTree::Ident(self.create(var_expr.stream()));
                }
            }
        }
    }

    pub fn get_tokens(&self) -> TokenStream {
        let var_names = (0..).map(|i| format!("$v{i}"));
        let var_values = &self.values;

        quote::quote!(serde_json::json!({
            #(#var_names: #var_values),*
        }))
    }
}
