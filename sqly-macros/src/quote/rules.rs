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



use super::*;

pub struct Build<'c, T> {
    table: &'c T,
    string: Option<String>,
    pending: Option<String>,
    checking: Option<String>,
    stream: Option<TokenStream>,
    optional: bool,
}

impl<'c, T: Builder> Build<'c, T> {
    pub fn new(table: &'c T) -> Result<Self> {
        let dynamic = table.dynamic().is_some();
        let checked = table.checked();
        let certain = table.certain();
        Ok(Self {
            table,
            pending: dynamic.then(String::new),
            checking: checked.then(String::new),
            string: (!dynamic).then(String::new),
            stream: dynamic.then(TokenStream::new),
            optional: !certain,
        })
    }
}

impl<'c, T: Builder> Build<'c, T> {

    fn idx(field: &T::Field) -> syn::Ident {
        quote::format_ident!("_{}", field.ident())
    }

    fn put(&mut self) -> Result<()> {
        if let Some(pending) = &mut self.pending {
            if let Some(stream) = &mut self.stream {
                if !pending.is_empty() {
                    stream.extend(quote::quote! {
                        query.push_str(#pending);
                    });
                }
            }
            pending.drain(..);
        }
        Ok(())
    }

    fn add(&mut self, add: TokenStream) -> Result<()> {
        self.put()?;
        if let Some(stream) = &mut self.stream {
            stream.extend(add);
        }
        Ok(())
    }

}

impl<'c, T: Builder> Build<'c, T> {

    pub fn str(&mut self, str: &str) -> Result<()> {
        if let Some(checking) = &mut self.checking {
            checking.push_str(str);
        }
        if let Some(pending) = &mut self.pending {
            pending.push_str(str);
        }
        if let Some(string) = &mut self.string {
            string.push_str(str);
        }
        Ok(())
    }

    pub fn duo<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(Target) -> Result<String>
    {
        if let Some(checking) = &mut self.checking {
            let mac = f(Target::Macro)?;
            checking.push_str(&mac);
        }
        let fun = f(Target::Function)?;
        if let Some(pending) = &mut self.pending {
            pending.push_str(&fun);
        }
        if let Some(string) = &mut self.string {
            string.push_str(&fun);
        }
        Ok(())
    }

    pub fn cut(&mut self, str: &[&str]) -> Result<bool> {
        if let Some(pending) = &mut self.pending {
            let mut cut = false;
            for str in str {
                if pending.ends_with(str) {
                    pending.truncate(pending.len() - str.len());
                    cut = true;
                    break;
                }
            }
            if !cut {
                let option = self.optional.then(|| {
                    quote::quote! { return ::core::option::Option::None; }
                });
                let len = str.iter().map(|str| str.len());
                self.add(quote::quote! {
                    #(if query.ends_with(#str) {
                        query.truncate(query.len() - #len);
                    } else)* { #option }
                })?;
            }
        }
        let mut res = true;
        if let Some(checking) = &mut self.checking {
            let mut cut = false;
            for str in str {
                if checking.ends_with(str) {
                    checking.truncate(checking.len() - str.len());
                    cut = true;
                    break;
                }
            }
            res = res && cut;
        }
        if let Some(string) = &mut self.string {
            let mut cut = false;
            for str in str {
                if string.ends_with(str) {
                    string.truncate(string.len() - str.len());
                    cut = true;
                    break;
                }
            }
            res = res && cut;
        }
        Ok(res)
    }

    pub fn opt<F>(&mut self, field: &T::Field, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        if self.table.optional(field).is_some() {
            self.put()?;
            if let Some(stream) = self.stream.take() {
                self.stream = Some(TokenStream::new());
                if let Err(err) = f(self) {
                    self.stream = Some(stream);
                    return Err(err);
                }
                self.put()?;
                let ident = Self::idx(field);
                let body = self.stream.replace(stream);
                self.stream.as_mut().map(|stream| stream.extend(quote::quote! {
                    if let ::core::option::Option::Some(_) = &#ident.value {
                        #body
                    }
                }));
                return Ok(());
            }
        }
        f(self)?;
        Ok(())
    }

    pub fn arg<K, V>(
        &mut self,
        params: &mut Params<K, V, &'c T::Field>,
        list: &[&Info<String>],
        mut cell: Option<&mut V>,
    ) -> Result<()>
    where
        K: Borrow<str> + Hash + Eq,
        V: Placer<&'c T::Field>,
    {
        let mut dst = String::new();
        if !list.is_empty() {
            params.output(&mut dst, list)?;
        } else if let Some(cell) = &mut cell {
            params.place(&mut dst, cell)?;
        } else { return Ok(()); }
        if let Some(checking) = &mut self.checking {
            checking.push_str(&dst);
        }
        if let Some(string) = &mut self.string {
            string.push_str(&dst);
        }
        if self.stream.is_some() {
            let mut fmt = Format::default();
            if !list.is_empty() {
                params.output(&mut fmt, list)?;
            } else if let Some(cell) = cell {
                params.place(&mut fmt, cell)?;
            } else { return Ok(()); }
            let (str, map) = fmt.into_inner();
            let tuple = map.values().map(|field| field.ident());
            let value = map.values().map(|field| Self::idx(field));
            let write = quote::quote! { ::core::write!(&mut query, #str).ok(); };
            let block = match map.len() {
                0 => quote::quote! { #write },
                _ => quote::quote! { {
                    let (#(#tuple,)*) = (#(#value.bind(&mut args),)*);
                    #write
                } }
            };
            self.add(block)?;
        }
        Ok(())
    }

}

impl<'c, T: Builder> Build<'c, T> {

    pub fn done(mut self, args: Vec<&'c T::Field>, rest: Vec<&'c T::Field>) -> Result<Done<'c, T>> {
        self.put()?;
        let stream = if let Some(stream) = &mut self.stream {
            let db = db![];
            let krate = self.table.krate()?;
            let option = self.optional.then(|| {
                quote::quote! { ::core::option::Option::Some }
            });
            let tuple = match args.len() {
                0 => quote::quote! { query },
                _ => quote::quote! { (query, args) },
            };
            let mut full = args.clone();
            full.extend(rest.iter().filter(|field| {
                self.table.optional(field).is_some()
            }));
            let expr = full.iter().map(|field| {
                self.table.value(field, Target::Function)
            }).collect::<Result<Vec<_>>>()?;
            let ident = full.iter().map(|field| Self::idx(field));
            let bind = (0..full.len()).map(|i| syn::Index::from(i));
            let args = (!args.is_empty()).then(|| quote::quote! {
                use ::core::fmt::Write as _;
                let args = <#db as #krate::sqlx::Database>::Arguments::default();
                let mut args = ::core::result::Result::Ok(args);
            });
            let arg = (!full.is_empty()).then(|| quote::quote! {
                let arg = (#(&(#expr),)*);
                #args
                #(let mut #ident = #krate::dynamic::Bind::new(arg.#bind);)*
            });
            quote::quote! {
                #arg
                let mut query = ::std::string::String::new();
                #stream
                #option(#tuple)
            }
        } else if let Some(string) = &mut self.string {
            let db = db![];
            let len = args.len();
            let krate = self.table.krate()?;
            let bind = (0..len).map(|i| {
                let i = syn::Index::from(i);
                quote::quote! { arg.#i }
            }).collect::<Vec<_>>();
            let expr = args.iter().map(|field| {
                self.table.value(field, Target::Function)
            }).collect::<Result<Vec<_>>>()?;
            (!args.is_empty()).then(|| quote::quote! {
                let arg = (#(&(#expr),)*);
                use #krate::sqlx::Arguments as _;
                let mut args = <#krate #db as #krate::sqlx::Database>::Arguments::default();
                args.reserve(#len, 0 #(+ #krate::sqlx::Encode::<#krate #db>::size_hint(#bind))*);
                let args = ::core::result::Result::Ok(args)
                #(.and_then(move |mut args| args.add(#bind).map(move |()| args)))*;
                (#string, args)
            }).unwrap_or_else(|| quote::quote! { #string })
        } else { TokenStream::new() };

        Ok(Done {
            query: self.string,
            check: self.checking,
            map: None,
            stream,
            args,
            rest,
        })
    }

}

pub struct Done<'c, T: Struct> {
    pub stream: TokenStream,
    pub query: Option<String>,
    pub check: Option<String>,
    pub args: Vec<&'c T::Field>,
    pub rest: Vec<&'c T::Field>,
    pub map: Option<&'c syn::Path>,
}



macro_rules! build {
    ($table:ident { $(fn $f:ident(&self$(, $v:ident: $t:ty)*) -> $r:ty;)* }) => {
        impl Builder for $table { $(fn $f(&self$(, $v: $t)*) -> $r { self.$f($($v),*) })* }
    };
    ($($t:tt)*) => {
        pub trait Builder: Struct { $($t)* }
        build!(DeleteTable { $($t)* });
        build!(InsertTable { $($t)* });
        build!(SelectTable { $($t)* });
        build!(UpdateTable { $($t)* });
    };
}

build! {
    fn checked(&self) -> bool;
    fn certain(&self) -> bool;
    fn dynamic(&self) -> Option<Span>;
    fn optional(&self, field: &Self::Field) -> Option<Span>;
    fn value(&self, field: &Self::Field, target: Target) -> Result<TokenStream>;
    fn krate(&self) -> Result<Cow<syn::Path>>;
}
