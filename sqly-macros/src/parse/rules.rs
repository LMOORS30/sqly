use proc_macro2::{TokenStream, TokenTree};
use proc_macro2::{Punct, Spacing};
use proc_macro2::Span;
use quote::ToTokens;

pub struct Void;

pub struct Info<T> {
    pub span: Option<Span>,
    pub data: T,
}

pub struct Name<T> {
    pub name: &'static str,
    pub span: Span,
    pub data: T,
}

pub mod safe {
    pub use super::Void;

    pub struct Info<T> {
        pub data: T,
    }

    pub struct Name<T> {
        pub name: &'static str,
        pub data: T,
    }
}



pub trait Safe: Sized {
    type Error;
    type Safe: Send + Sync + 'static;
    fn send(self: &Self) -> Result<Self::Safe, Self::Error>;
    fn sync(safe: &Self::Safe) -> Result<Self, Self::Error>;
}

pub trait Save {}

impl Save for syn::Expr {}
impl Save for syn::Type {}
impl Save for syn::Path {}
impl Save for syn::Ident {}
impl Save for syn::Generics {}
impl Save for syn::Visibility {}



pub trait Spany {
    fn spany(&self) -> Option<Span>;
}

pub trait Infos<T> {
    fn infos(&self) -> Vec<&Info<T>>;
}

impl<T> Spany for Option<Name<T>> {
    fn spany(&self) -> Option<Span> {
        self.as_ref().map(|opt| opt.span)
    }
}
impl<T> Spany for Vec<Name<T>> {
    fn spany(&self) -> Option<Span> {
        self.first().map(|opt| opt.span)
    }
}

impl<T> Infos<T> for Vec<Name<Info<T>>> {
    fn infos(&self) -> Vec<&Info<T>> {
        self.iter().map(|list| &list.data).collect()
    }
}

impl<T> Infos<T> for Vec<Name<Vec<Info<T>>>> {
    fn infos(&self) -> Vec<&Info<T>> {
        self.iter().flat_map(|list| &list.data).collect()
    }
}

macro_rules! spany {
    ($($opt:expr),+ $(,)?) => { [$($opt.spany()),*].into_iter().find_map(|opt| opt) }
}



pub trait Spanned {
    fn spanned(&self) -> Span;
}

impl<T: syn::spanned::Spanned> Spanned for T {
    fn spanned(&self) -> Span {
        self.span()
    }
}

impl<T: syn::parse::Parse> syn::parse::Parse for Info<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Info { data: T::parse(input)?, span: None })
    }
}

impl<T: Spanned> Info<T> {
    pub fn span(&self) -> Span {
        match self.span {
            Some(span) => span,
            None => self.data.spanned(),
        }
    }
}



pub fn respanned(span: Span, stream: TokenStream) -> TokenStream {
    respan(stream, span).collect()
}

fn respan(stream: TokenStream, span: Span) -> impl Iterator<Item = TokenTree> {
    fn respanned(mut token: TokenTree, span: Span) -> TokenTree {
        if let TokenTree::Group(g) = &mut token {
            let delimiter = g.delimiter();
            let stream = respan(g.stream(), span).collect();
            *g = proc_macro2::Group::new(delimiter, stream);
        }
        token.set_span(span);
        token
    }
    stream.into_iter().map(move |token| respanned(token, span))
}

impl<T: ToTokens> ToTokens for Info<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.span {
            Some(span) => {
                let data = self.data.to_token_stream();
                tokens.extend(respan(data, span));
            }
            None => self.data.to_tokens(tokens),
        }
    }
}



macro_rules! asref {
($($c:tt)?) => {

impl ToTokens for Name<$($c)?Info<Void>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut name = quote::format_ident!("{}", self.name);
        name.set_span(self.span);
        name.to_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Name<$($c)?Info<T>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut name = quote::format_ident!("{}", self.name);
        let equals = Punct::new('=', Spacing::Alone);
        name.set_span(self.span);
        name.to_tokens(tokens);
        equals.to_tokens(tokens);
        self.data.to_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Name<$($c)?Option<T>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut name = quote::format_ident!("{}", self.name);
        let equals = Punct::new('=', Spacing::Alone);
        name.set_span(self.span);
        name.to_tokens(tokens);
        if let Some(data) = self.data.as_ref() {
            equals.to_tokens(tokens);
            data.to_tokens(tokens);
        }
    }
}

impl<T: ToTokens> ToTokens for Name<$($c)?Vec<T>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut name = quote::format_ident!("{}", self.name);
        let mut char = Punct::new('=', Spacing::Alone);
        name.set_span(self.span);
        name.to_tokens(tokens);
        for data in self.data.as_slice() {
            char.to_tokens(tokens);
            data.to_tokens(tokens);
            char = Punct::new(',', Spacing::Alone);
        }
    }
}

} }

asref!();
asref!(&);

impl<T> Name<T> {
    pub fn rename(&self, name: &'static str) -> Name<&T> {
        Name {
            data: &self.data,
            span: self.span,
            name,
        }
    }
}



impl<T> Info<T> {
    pub fn send<S>(&self, data: S) -> syn::Result<safe::Info<S>> {
        Ok(safe::Info { data })
    }
}

impl<S> safe::Info<S> {
    pub fn sync<T>(&self, data: T) -> syn::Result<Info<T>> {
        Ok(Info { span: Some(Span::call_site()), data })
    }
}

impl<T> Name<T> {
    pub fn send<S>(&self, data: S) -> syn::Result<safe::Name<S>> {
        Ok(safe::Name { name: self.name, data })
    }
}

impl<S> safe::Name<S> {
    pub fn sync<T>(&self, data: T) -> syn::Result<Name<T>> {
        Ok(Name { name: self.name, span: Span::call_site(), data })
    }
}

impl<T: quote::ToTokens + syn::parse::Parse + Save> Safe for T {
    type Safe = String;
    type Error = syn::Error;
    fn send(self: &Self) -> Result<Self::Safe, Self::Error> {
        Ok(self.to_token_stream().to_string())
    }
    fn sync(safe: &Self::Safe) -> Result<Self, Self::Error> {
        syn::parse(safe.parse().map_err(|err| {
            let span = Span::call_site();
            let msg = format!("{}", err);
            syn::Error::new(span, msg)
        })?)
    }
}



#[allow(unused)]
pub trait Struct {
    type Field: Field;
    fn ident(&self) -> &syn::Ident;
    fn fields(&self) -> &Vec<Self::Field>;
    fn vis(&self) -> &syn::Visibility;
    fn generics(&self) -> &syn::Generics;
}

#[allow(unused)]
pub trait Field {
    fn ty(&self) -> &syn::Type;
    fn ident(&self) -> &syn::Ident;
    fn vis(&self) -> &syn::Visibility;
}

macro_rules! parse {
{ $t:vis $table:ident { $(($($a:tt)*),)* } $f:vis $field:ident { $(($($b:tt)*),)* } } => {
paste::paste! {

    safe! {
        $t struct $table {
            pub ident: syn::Ident,
            pub fields: $field [*],
            pub vis: syn::Visibility,
            pub attr: [<$table Attr>] [Box],
            pub generics: syn::Generics,
        }
    }

    safe! {
        $f struct $field {
            pub ty: syn::Type,
            pub ident: syn::Ident,
            pub vis: syn::Visibility,
            pub attr: [<$field Attr>],
        }
    }

    impl crate::parse::rules::Struct for $table {
        type Field = $field;
        fn ident(&self) -> &syn::Ident { &self.ident }
        fn fields(&self) -> &Vec<Self::Field> { &self.fields }
        fn vis(&self) -> &syn::Visibility { &self.vis }
        fn generics(&self) -> &syn::Generics { &self.generics }
    }

    impl crate::parse::rules::Field for $field {
        fn ty(&self) -> &syn::Type { &self.ty }
        fn ident(&self) -> &syn::Ident { &self.ident }
        fn vis(&self) -> &syn::Visibility { &self.vis }
    }

    attr!($t [<$table Attr>] { $(($($a)*),)* });
    attr!($f [<$field Attr>] { $(($($b)*),)* });

    impl TryFrom<syn::DeriveInput> for $table {
        type Error = syn::Error;

        fn try_from(input: syn::DeriveInput) -> syn::Result<Self> {
            let data = match input.data {
                syn::Data::Struct(data) => {
                    if data.fields.iter().all(|f| {
                        f.ident.is_some()
                    }) { Some(data) }
                    else { None }
                }
                _ => None,
            };

            let data = match data {
                Some(data) => data,
                None => {
                    let span = Span::call_site();
                    let msg = "not a struct with named fields";
                    return Err(syn::Error::new(span, msg));
                }
            };

            let span = Span::call_site();
            let info = (input.attrs, span);
            let attr = Box::new([<$table Attr>]::try_from(info)?);
            let fields = data.fields.into_iter().map($field::try_from);
            let fields = fields.collect::<syn::Result<Vec<$field>>>()?;

            Ok(Self {
                attr,
                fields,
                generics: input.generics,
                ident: input.ident,
                vis: input.vis,
            })
        }
    }

    impl TryFrom<syn::Field> for $field {
        type Error = syn::Error;

        fn try_from(field: syn::Field) -> syn::Result<Self> {
            let ident = field.ident.expect("unnamed field");

            let span = ident.span();
            let info = (field.attrs, span);
            let attr = [<$field Attr>]::try_from(info)?;

            Ok(Self {
                attr,
                ident,
                ty: field.ty,
                vis: field.vis,
            })
        }
    }

} } }



macro_rules! attr {
($vis:vis $i:ident { $((($n:ident $(as $s:literal)?)$r:tt $($t:tt)*),)* }) => {
paste::paste! {

    #[allow(non_camel_case_types)]
    enum [<$i Enum>] {
        $($n(syncd!(crate::parse::rules::<! $($t)*>)),)*
    }

    $vis struct $i {
        $(pub $n: syncd!(crate::parse::rules::<$r $($t)*>),)*
    }

    $vis struct [<Safe $i>] {
        $(pub $n: saved!(crate::parse::rules::safe::<$r $($t)*>),)*
    }

    impl syn::parse::Parse for [<$i Enum>] {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let ident = input.call(<syn::Ident as syn::ext::IdentExt>::parse_any)?;
            let span = ident.span();
            match &*ident.to_string() {
                $(named!($n $($s)?) => token!(input, $($t)*).map(|v| {
                    [<$i Enum>]::$n(crate::parse::rules::Name {
                        name: named!($n $($s)?),
                        data: v,
                        span,
                    })
                }),)*
                n => Err(syn::Error::new(span, format!(
                    "unknown attribute: #[sqly({})]", n
                ))),
            }
        }
    }

    impl TryFrom<(Vec<syn::Attribute>, Span)> for $i {
        type Error = syn::Error;

        fn try_from((data, _span): (Vec<syn::Attribute>, Span)) -> syn::Result<Self> {
            let iter = data.into_iter().filter(|a| a.path().is_ident("sqly")).map(|a| {
                type Separate<T> = syn::punctuated::Punctuated::<T, syn::Token![,]>;
                a.parse_args_with(Separate::<[<$i Enum>]>::parse_terminated)
            }).collect::<syn::Result<Vec<_>>>()?.into_iter().flatten();

            $(let mut $n = r#match!({ $r } {
                ! => Option::None,
                ? => Option::None,
                + => Vec::new(),
                * => Vec::new(),
            });)*

            for attr in iter {
                match attr {
                    $([<$i Enum>]::$n(v) => {
                        r#match!({ $r } {
                            ! => match $n { None => $n = Some(v), Some(_) => {
                                return Err(syn::Error::new(v.span, format!(
                                    "duplicate attribute: #[sqly({})]", named!($n $($s)?)
                                )));
                            } },
                            ? => ~ !,
                            + => r#match!({ $($t)* } {
                                ! => $n.push(v),
                                ? => ~ !,
                                + => {
                                    let first: Option<&crate::parse::rules::Name<Vec<_>>> = $n.first();
                                    if first.is_some_and(|w| w.data.is_empty() || v.data.is_empty()) {
                                        return Err(syn::Error::new(v.span, format!(
                                            "duplicate attribute: #[sqly({})]", named!($n $($s)?)
                                        )));
                                    }
                                    $n.push(v);
                                },
                                * => ~ +,
                            }),
                            * => ~ +,
                        })
                    },)*
                }
            }

            $(let $n = r#match!({ $r } {
                ! => $n,
                ? => $n,
                + => if $n.is_empty() { None } else { Some($n) },
                * => $n,
            });)*

            $(let $n = r#match!({ $r } {
                ! => match $n { Some(n) => n, None => {
                    return Err(syn::Error::new(_span, format!(
                        "missing attribute: #[sqly({})]", named!($n $($s)?)
                    )));
                } },
                ? => $n,
                + => ~ !,
                * => $n,
            });)*

            Ok($i { $($n,)* })
        }
    }

    impl crate::parse::rules::Safe for $i {
        type Safe = [<Safe $i>];
        type Error = syn::Error;
        fn send(self: &Self) -> core::result::Result<Self::Safe, Self::Error> {
            Ok(Self::Safe { $($n: safe!({ $r }, self.$n, |name: &crate::parse::rules::Name<_>| {
                name.send(safe!({ $($t)* }, name.data, |info: &crate::parse::rules::Info<_>| {
                    info.send(safe!({ $($t)* }: info.data, crate::parse::rules::Safe::send))
                })?)
            })?,)* })
        }
        fn sync(safe: &Self::Safe) -> core::result::Result<Self, Self::Error> {
            Ok(Self { $($n: safe!({ $r }, safe.$n, |name: &crate::parse::rules::safe::Name<_>| {
                name.sync(safe!({ $($t)* }, name.data, |info: &crate::parse::rules::safe::Info<_>| {
                    info.sync(safe!({ $($t)* }: info.data, crate::parse::rules::Safe::sync))
                })?)
            })?,)* })
        }
    }

} } }



macro_rules! vars {
{ $($vis:vis $e:ident$(:)? $($a:ident),* { $(($($t:tt)*),)* })* } => {
    $(vari! { $vis $e: $($a,)* { $(($($t)*),)* } })*
} }

macro_rules! vari {
{ $vis:vis $e:ident: $($a:ident,)* { $(($v:ident = $n:literal),)* } } => {
    vari! { impl $vis $e { $(($v, stringify!($n), $n),)* } true }
    vari! { $e: $($a,)* = { $($v,)* } }
};
{ $vis:vis $e:ident: $($a:ident,)* { $(($v:ident = $n:ident),)* } } => {
    vari! { impl $vis $e { $(($v, stringify!($n), stringify!($n)),)* } false }
    vari! { $e: $($a,)* = { $($v,)* } }
};
{ $e:ident: $($a:ident,)* = $v:tt } => {
    $(vari! { impl $e: $a $v })*
};
{ impl $vis:vis $e:ident { $(($v:ident, $n:expr, $s:expr),)* } $lit:tt } => {

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    $vis enum $e {
        $($v,)*
    }

    impl syn::parse::Parse for crate::parse::rules::Info<$e> {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let (value, span) = r#bool!({ $lit } {
                true => input.parse().map(|s: syn::LitStr| (s.value(), s.span()))?,
                false => {
                    let value = input.call(<syn::Ident as syn::ext::IdentExt>::parse_any)?;
                    (value.to_string(), value.span())
                },
            });
            match value.as_str() {
                $($s => Ok(crate::parse::rules::Info { data: $e::$v, span: Some(span) }),)*
                n => {
                    let list = [$($n,)*].join(", ");
                    Err(syn::Error::new(span, format!(
                        r#bool!({ $lit } {
                            false => "unknown variant: {}\n must be one of: {}",
                            true => "unknown variant: \"{}\"\n must be one of: {}",
                        }), n, list
                    )))
                }
            }
        }
    }

    impl quote::ToTokens for $e {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                $($e::$v => r#bool!({ $lit } {
                    false => quote::format_ident!("{}", $s).to_tokens(tokens),
                    true => $s.to_tokens(tokens),
                }),)*
            }
        }
    }

    impl std::fmt::Display for $e {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                $($e::$v => r#bool!({ $lit } {
                    false => write!(f, "{}", $s),
                    true => write!(f, "\"{}\"", $s),
                }),)*
            }
        }
    }

};
{ impl $e:ident: $a:ident { $($v:ident,)* } } => {
    impl From<$e> for $a {
        fn from(e: $e) -> Self {
            match e {
                $($e::$v => Self::$v,)*
            }
        }
    }
    impl TryFrom<$a> for $e {
        type Error = $a;
        #[allow(unreachable_patterns)]
        fn try_from(a: $a) -> core::result::Result<Self, $a> {
            match a {
                $(Self::Error::$v => Ok(Self::$v),)*
                _ => Err(a),
            }
        }
    }
};
{ $vis:vis $e:ident: { $(($v:ident: $($t:tt)*),)* } } => {
paste::paste! {

    #[derive(Clone)]
    $vis enum $e {
        $($v(syncd!($($t)*)),)*
    }

    #[derive(Clone)]
    $vis enum [<Safe $e>] {
        $($v(saved!($($t)*)),)*
    }

    impl syn::parse::Parse for crate::parse::rules::Info<$e> {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let span = input.span();
            Err(syn::Error::new(span, ""))
            $(.or_else(|_| token!(input, $($t)*).map(|info| {
                crate::parse::rules::Info { data: $e::$v(info.data), span: info.span }
            })))*
            .or_else(|_| {
                let list = [$(stringify!($($t)*),)*];
                let mut list = list.iter().map(|name| {
                    match name.rfind(':') {
                        Some(i) => &name[i+1..],
                        None => name,
                    }
                }).collect::<Vec<_>>();
                let pop = list.pop().unwrap_or("");
                let mut list = list.join(", ");
                if !list.is_empty() {
                    list.push_str(" or ");
                }
                list.push_str(pop);
                let mut msg = format!("expected {}", list);
                if input.is_empty() {
                    msg = format!("unexpected end of input, {}", msg);
                }
                Err(syn::Error::new(span, msg))
            })
        }
    }

    impl quote::ToTokens for $e {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                $($e::$v(v) => quote::ToTokens::to_tokens(v, tokens),)*
            }
        }
    }

    impl crate::parse::rules::Safe for $e {
        type Safe = [<Safe $e>];
        type Error = syn::Error;
        fn send(self: &Self) -> core::result::Result<Self::Safe, Self::Error> {
            Ok(match self { $($e::$v(v) => [<Safe $e>]::$v(saved!(v, $($t)*)),)* })
        }
        fn sync(safe: &Self::Safe) -> core::result::Result<Self, Self::Error> {
            Ok(match safe { $([<Safe $e>]::$v(v) => $e::$v(syncd!(v, $($t)*)),)* })
        }
    }

} } }



macro_rules! safe {
    { $vis:vis struct $i:ident { $($v:vis $n:ident: $t:ty $([$r:tt])?,)* } } => {
    paste::paste! {
        $vis struct $i {
            $($v $n: safe!($t, $t $([$r])?),)*
        }

        $vis struct [<Safe $i>] {
            $($v $n: safe!($t, <$t as crate::parse::rules::Safe>::Safe $([$r])?),)*
        }

        impl $i {
            pub fn send(&self) -> syn::Result<[<Safe $i>]> {
                <$i as crate::parse::rules::Safe>::send(self)
            }
        }

        impl [<Safe $i>] {
            pub fn sync(&self) -> syn::Result<$i> {
                <$i as crate::parse::rules::Safe>::sync(self)
            }
        }

        impl crate::parse::rules::Safe for $i {
            type Safe = [<Safe $i>];
            type Error = syn::Error;
            fn send(self: &Self) -> core::result::Result<Self::Safe, Self::Error> {
                Ok(Self::Safe { $($n: safe!({ $($r)? }, self.$n, crate::parse::rules::Safe::send)?,)* })
            }
            fn sync(safe: &Self::Safe) -> core::result::Result<Self, Self::Error> {
                Ok(Self { $($n: safe!({ $($r)? }, safe.$n, crate::parse::rules::Safe::sync)?,)* })
            }
        }
    } };
    ({ }: $data:expr, $spec:path) => ( crate::parse::rules::safe::Void );
    ({ (= $($t:tt)*)$_:tt }: $data:expr, $spec:path) => ( specd!($($t)*, &$data, $spec) );
    ({ = }, $val:expr, $_:expr) => { core::result::Result::<_, Self::Error>::Ok($val) };
    ({ Box }, $val:expr, $fun:expr) => { $fun(&*$val).map(Box::new) };
    ({ $($t:tt)* }, $val:expr, $fun:expr) => { r#match!({ $($t)* } {
        ! => $fun(&$val),
        ? => $val.as_ref().map($fun).map_or(Ok(None), |ok| ok.map(Some)),
        + => $val.iter().map($fun).collect::<Result<Vec<_>>>(),
        * => ~ +,
    }) };
    ($_:ty, $t:ty) => ( $t );
    ($_:ty, $t:ty [!]) => ( $t );
    ($_:ty, $t:ty [Box]) => ( Box<$t> );
    ($_:ty, $t:ty [?]) => ( Option<$t> );
    ($_:ty, $t:ty [+]) => ( Vec<$t> );
    ($_:ty, $t:ty [*]) => ( Vec<$t> );
    ($t:ty, $_:ty [=]) => ( $t );
}



macro_rules! token {
    ($i:ident, (= $($t:tt)*)*) => ({
        match $i.peek(syn::Token![=]) {
            true => token!($i, (= $($t)*)+),
            false => Ok(Vec::new()),
        }
    });
    ($i:ident, (= $($t:tt)*)+) => ({
        $i.parse::<syn::Token![=]>()?;
        let mut vec = Vec::new();
        vec.push(token!($i, $($t)*)?);
        while $i.peek(syn::Token![,]) {
            if $i.peek3(syn::Token![=]) {
                break;
            }
            token!(lit($($t)*) {
                if $i.peek2(<syn::Ident as syn::ext::IdentExt>::peek_any) {
                    break;
                }
            });
            $i.parse::<syn::Token![,]>()?;
            if $i.is_empty() {
                break;
            }
            vec.push(token!($i, $($t)*)?);
        }
        Ok(vec)
    });
    ($i:ident, (= $($t:tt)*)?) => ({
        match $i.peek(syn::Token![=]) {
            true => token!($i, (= $($t)*)!).map(Some),
            false => Ok(None),
        }
    });
    ($i:ident, (= $($t:tt)*)!) => ({
        $i.parse::<syn::Token![=]>()?;
        token!($i, $($t)*)
    });
    ($i:ident,) => ( Ok(crate::parse::rules::Info { data: crate::parse::rules::Void, span: None }) );
    ($i:ident, f64) => ( token!(let $i: syn::LitFloat = $i.base10_parse()?) );
    ($i:ident, u64) => ( token!(let $i: syn::LitInt = $i.base10_parse()?) );
    ($i:ident, i64) => ( token!(let $i: syn::LitInt = $i.base10_parse()?) );
    ($i:ident, String) => ( token!(let $i: syn::LitStr = $i.value()) );
    ($i:ident, bool) => ( token!(let $i: syn::LitBool = $i.value()) );
    ($i:ident, $($t:tt)+) => ( $i.parse::<crate::parse::rules::Info<syncd!($($t)+)>>() );
    (let $i:ident: $t:ty = $v:expr) => ( $i.parse::<$t>().and_then(|$i: $t| {
        Ok(crate::parse::rules::Info { span: Some($i.span()), data: $v })
    }) );
    (lit(f64) { $($t:tt)* }) => ( $($t)* );
    (lit(u64) { $($t:tt)* }) => ( $($t)* );
    (lit(i64) { $($t:tt)* }) => ( $($t)* );
    (lit(String) { $($t:tt)* }) => ( $($t)* );
    (lit($($t:tt)+) { $($_:tt)* }) => ();
}

macro_rules! specd {
    (syn::$t:ident, $data:expr, $spec:path) => ( $spec($data)? );
    (safe::$t:ident, $data:expr, $spec:path) => ( $spec($data)? );
    ($($t:ty)?, $data:expr, $spec:path) => ( Clone::clone($data) );
}

macro_rules! saved {
    ($($p:ident::)*<$a:tt>) => ( typed!($($p::)*<$a>) );
    ($($p:ident::)*<$a:tt (= $($t:tt)*)$b:tt>) => ( typed!($($p::)*<$a(= saved!($($t)*))$b>) );
    ($data:expr, $($t:tt)*) => ( specd!($($t)*, $data, crate::parse::rules::Safe::send) );
    (syn::$t:ident) => ( <syn::$t as crate::parse::rules::Safe>::Safe );
    (safe::$t:ident) => ( <$t as crate::parse::rules::Safe>::Safe );
    ($($t:ty)?) => ( $($t)? );
}

macro_rules! syncd {
    ($($p:ident::)*<$a:tt>) => ( typed!($($p::)*<$a>) );
    ($($p:ident::)*<$a:tt (= $($t:tt)*)$b:tt>) => ( typed!($($p::)*<$a(= syncd!($($t)*))$b>) );
    ($data:expr, $($t:tt)*) => ( specd!($($t)*, $data, crate::parse::rules::Safe::sync) );
    (syn::$t:ident) => ( syn::$t );
    (safe::$t:ident) => ( $t );
    ($($t:ty)?) => ( $($t)? );
}

macro_rules! typed {
    ($($p:ident::)*<?>) => ( Option<$($p::)*Name<$($p::)*Info<$($p::)*Void>>> );
    ($($p:ident::)*<*>) => ( Vec<$($p::)*Name<$($p::)*Info<$($p::)*Void>>> );
    ($($p:ident::)*<+>) => ( Vec<$($p::)*Name<$($p::)*Info<$($p::)*Void>>> );
    ($($p:ident::)*<!>) => ( $($p::)*Name<$($p::)*Info<$($p::)*Void>> );
    ($($p:ident::)*<? (= $t:ty)?>) => ( Option<$($p::)*Name<Option<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<? (= $t:ty)*>) => ( Option<$($p::)*Name<Vec<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<? (= $t:ty)+>) => ( Option<$($p::)*Name<Vec<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<? (= $t:ty)!>) => ( Option<$($p::)*Name<$($p::)*Info<$t>>> );
    ($($p:ident::)*<* (= $t:ty)?>) => ( Vec<$($p::)*Name<Option<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<* (= $t:ty)*>) => ( Vec<$($p::)*Name<Vec<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<* (= $t:ty)+>) => ( Vec<$($p::)*Name<Vec<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<* (= $t:ty)!>) => ( Vec<$($p::)*Name<$($p::)*Info<$t>>> );
    ($($p:ident::)*<+ (= $t:ty)?>) => ( Vec<$($p::)*Name<Option<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<+ (= $t:ty)*>) => ( Vec<$($p::)*Name<Vec<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<+ (= $t:ty)+>) => ( Vec<$($p::)*Name<Vec<$($p::)*Info<$t>>>> );
    ($($p:ident::)*<+ (= $t:ty)!>) => ( Vec<$($p::)*Name<$($p::)*Info<$t>>> );
    ($($p:ident::)*<! (= $t:ty)?>) => ( $($p::)*Name<Option<$($p::)*Info<$t>>> );
    ($($p:ident::)*<! (= $t:ty)*>) => ( $($p::)*Name<Vec<$($p::)*Info<$t>>> );
    ($($p:ident::)*<! (= $t:ty)+>) => ( $($p::)*Name<Vec<$($p::)*Info<$t>>> );
    ($($p:ident::)*<! (= $t:ty)!>) => ( $($p::)*Name<$($p::)*Info<$t>> );
}

macro_rules! r#match {
    ({ } { $($t:tt)* }) => ( r#type!(! { $($t)* }) );
    ({ ! } { $($t:tt)* }) => ( r#type!(! { $($t)* }) );
    ({ ? } { $($t:tt)* }) => ( r#type!(? { $($t)* }) );
    ({ + } { $($t:tt)* }) => ( r#type!(+ { $($t)* }) );
    ({ * } { $($t:tt)* }) => ( r#type!(* { $($t)* }) );
    ({ (= $_:ty)! } { $($t:tt)* }) => ( r#type!(! { $($t)* }) );
    ({ (= $_:ty)? } { $($t:tt)* }) => ( r#type!(? { $($t)* }) );
    ({ (= $_:ty)+ } { $($t:tt)* }) => ( r#type!(+ { $($t)* }) );
    ({ (= $_:ty)* } { $($t:tt)* }) => ( r#type!(* { $($t)* }) );
}

macro_rules! r#type {
    ($z:tt {
        ! => $(~ $a_:tt)? $($a:expr)?,
        ? => $(~ $b_:tt)? $($b:expr)?,
        + => $(~ $c_:tt)? $($c:expr)?,
        * => $(~ $d_:tt)? $($d:expr)?,
    }) => ( r#type!($z,
        r#type!($($a_)? $($a)?, $($a)?, $($b)?, $($c)?, $($d)?,),
        r#type!($($b_)? $($b)?, $($a)?, $($b)?, $($c)?, $($d)?,),
        r#type!($($c_)? $($c)?, $($a)?, $($b)?, $($c)?, $($d)?,),
        r#type!($($d_)? $($d)?, $($a)?, $($b)?, $($c)?, $($d)?,),
    ) );
    (!, $a:expr, $($b:expr)?, $($c:expr)?, $($d:expr)?,) => { $a };
    (?, $($a:expr)?, $b:expr, $($c:expr)?, $($d:expr)?,) => { $b };
    (+, $($a:expr)?, $($b:expr)?, $c:expr, $($d:expr)?,) => { $c };
    (*, $($a:expr)?, $($b:expr)?, $($c:expr)?, $d:expr,) => { $d };
    ($e:expr, $($_:tt)*) => { $e };
}

macro_rules! r#bool {
    ({ $z:tt } {
        true => $t:expr,
        false => $f:expr,
    }) => ( r#bool!($z, $t, $f) );
    ({ $z:tt } {
        false => $f:expr,
        true => $t:expr,
    }) => ( r#bool!($z, $t, $f) );
    (true, $t:expr, $f:expr) => { $t };
    (false, $t:expr, $f:expr) => { $f };
}

macro_rules! named {
    ($n:ident) => ( stringify!($n) );
    ($n:ident $s:literal) => ( $s );
}
