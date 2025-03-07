#[cfg(feature = "postgres")]
macro_rules! db { [] => ( quote::quote! { ::sqlx::Postgres } ) }



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
