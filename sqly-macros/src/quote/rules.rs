#[cfg(feature = "postgres")]
macro_rules! db { [] => ( quote::quote!{ ::sqlx::Postgres } ) }

macro_rules! result {
[$l:lifetime] => ({
    let db = db![];
    quote::quote!{
        ::sqlx::query::Query<
            $l,
            #db,
            <#db as ::sqlx::Database>::Arguments<$l>,
        >
    }
});
[$l:lifetime, $t:ident] => ({
    let db = db![];
    quote::quote!{
        ::sqlx::query::Map<
            $l,
            #db,
            fn(<#db as ::sqlx::Database>::Row) -> ::sqlx::Result<#$t>,
            <#db as ::sqlx::Database>::Arguments<$l>,
        >
    }
});
}



macro_rules! fun {
($query:ident, $args:ident) => ({
    let db = db![];
    let args = $args;
    let len = args.len();
    let bind = (0..len).map(|i| {
        quote::format_ident!("arg{i}")
    }).collect::<Vec<_>>();
    quote::quote!{
        use ::sqlx::Arguments;
        #(let #bind = &#args;)*
        let mut args = <#db as ::sqlx::Database>::Arguments::<'_>::default();
        args.reserve(#len, 0 #(+ ::sqlx::Encode::<#db>::size_hint(#bind))*);
        let args = ::core::result::Result::<_, ::sqlx::error::BoxDynError>::Ok(args)
        #(.and_then(move |mut args| args.add(#bind).map(move |()| args) ))*;
        ::sqlx::__query_with_result::<#db, _>(#$query, args)
    }
}) }



macro_rules! vectok {
[$($vec:expr,)*] => (
    [$(quote::ToTokens::to_token_stream(&$vec),)*].into_iter().filter(|v| {
        !proc_macro2::TokenStream::is_empty(v)
    }).collect::<Vec<_>>()
) }
