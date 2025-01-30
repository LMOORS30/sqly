#[cfg(feature = "postgres")]
macro_rules! db { [] => ( quote::quote! { ::sqlx::Postgres } ) }

macro_rules! result {
[$l:lifetime] => ({
    let db = db![];
    quote::quote! {
        ::sqlx::query::Query<
            $l,
            #db,
            <#db as ::sqlx::Database>::Arguments<$l>,
        >
    }
});
[$l:lifetime, $t:ident] => ({
    let db = db![];
    quote::quote! {
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
($table:ident, $query:ident, $args:ident) => ({
    let db = db![];
    let len = $args.len();
    let bind = (0..len).map(|i| {
        let i = syn::Index::from(i);
        quote::quote!{ arg.#i }
    }).collect::<Vec<_>>();
    let expr = $args.iter().map(|field| {
        $table.value(field, Target::Function)
    }).collect::<Result<Vec<_>>>()?;
    quote::quote! {
        let arg = (#(&(#expr),)*);
        use ::sqlx::Arguments as _;
        let mut args = <#db as ::sqlx::Database>::Arguments::default();
        args.reserve(#len, 0 #(+ ::sqlx::Encode::<#db>::size_hint(#bind))*);
        let args = ::core::result::Result::Ok(args)
        #(.and_then(move |mut args| args.add(#bind).map(move |()| args)))*;
        ::sqlx::__query_with_result(#$query, args)
    }
}) }



macro_rules! vectok {
[$($vec:expr),* $(,)?] => (
    [$(quote::ToTokens::to_token_stream(&($vec))),*].into_iter().filter(|v| {
        !proc_macro2::TokenStream::is_empty(v)
    }).collect::<Vec<_>>()
) }

macro_rules! args {
($vec:expr, [$(($name:ident = $($add:expr),* $(,)?)),* $(,)?]) => ({
    let vec = &mut ($vec);
    $(let name = stringify!($name);
    None$(.or_else(|| {
        let add = &($add);
        add.spany().map(|_| vec.extend(add.iter().map(|add| {
            quote::ToTokens::to_token_stream(&add.rename(name))
        })))
    }))*;)*
}) }
