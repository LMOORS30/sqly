#[cfg(feature = "postgres")]
macro_rules! result {
[$l:lifetime] => ( quote::quote!{ ::sqlx::query::Query<$l, ::sqlx::Postgres, ::sqlx::postgres::PgArguments> } );
}

#[cfg(feature = "unchecked")]
macro_rules! fun {
($query:ident, $args:ident) => ( quote::quote!{ ::sqlx::query(#$query)#(.bind(&#$args))* } );
}

#[cfg(not(feature = "unchecked"))]
macro_rules! fun {
($query:ident, $args:ident) => ( quote::quote!{ ::sqlx::query!(#$query #(, &#$args)*) } );
}



macro_rules! vectok {
[$($vec:expr,)*] => (
    [$(quote::ToTokens::to_token_stream(&$vec),)*].into_iter().filter(|v| {
        !proc_macro2::TokenStream::is_empty(v)
    }).collect::<Vec<_>>()
) }
