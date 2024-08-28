use proc_macro2::Span;

#[derive(Clone, Debug)]
pub struct Empty;

#[derive(Clone, Debug)]
pub struct Info<T>{
    pub span: Span,
    pub data: T,
}

#[derive(Clone, Debug)]
pub struct Name<T>{
    pub name: &'static str,
    pub span: Span,
    pub data: T,
}

impl<T> Info<T> {
    pub fn new(data: T, span: Span) -> Self {
        Self {
            data,
            span,
        }
    }
}

impl<T> Name<T> {
    pub fn new(info: Info<T>, name: &'static str) -> Self {
        Self {
            data: info.data,
            span: info.span,
            name,
        }
    }
}



impl<T: quote::ToTokens> quote::ToTokens for Info<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let data = self.data.to_token_stream();
        tokens.extend(data.into_iter().map(|mut tt| {
            tt.set_span(self.span);
            tt
        }))
    }
}

impl quote::ToTokens for Name<Info<Empty>> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut name = quote::format_ident!("{}", self.name);
        name.set_span(self.span);
        name.to_tokens(tokens);
    }
}

impl<T: quote::ToTokens> quote::ToTokens for Name<Info<T>> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let equals = proc_macro2::Punct::new('=', proc_macro2::Spacing::Alone);
        let mut name = quote::format_ident!("{}", self.name);
        name.set_span(self.span);
        name.to_tokens(tokens);
        equals.to_tokens(tokens);
        self.data.to_tokens(tokens);
    }
}

impl<T: quote::ToTokens> quote::ToTokens for Name<Option<T>> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let equals = proc_macro2::Punct::new('=', proc_macro2::Spacing::Alone);
        let mut name = quote::format_ident!("{}", self.name);
        name.set_span(self.span);
        name.to_tokens(tokens);
        if let Some(data) = self.data.as_ref() {
            equals.to_tokens(tokens);
            data.to_tokens(tokens);
        }
    }
}

impl<T: quote::ToTokens> quote::ToTokens for Name<Vec<T>> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut char = proc_macro2::Punct::new('=', proc_macro2::Spacing::Alone);
        let mut name = quote::format_ident!("{}", self.name);
        name.set_span(self.span);
        name.to_tokens(tokens);
        for data in &self.data {
            char.to_tokens(tokens);
            data.to_tokens(tokens);
            char = proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone);
        }
    }
}



macro_rules! parse {
{ $t:vis $table:ident { $(($($a:tt)*),)* } $f:vis $field:ident { $(($($b:tt)*),)* } } => {
paste::paste! {

    $t struct $table {
        pub ident: syn::Ident,
        pub fields: Vec<$field>,
        pub vis: syn::Visibility,
        pub attr: [<$table Attr>],
        pub generic: syn::Generics,
    }

    $f struct $field {
        pub ty: syn::Type,
        pub ident: syn::Ident,
        pub vis: syn::Visibility,
        pub attr: [<$field Attr>],
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
                },
                _ => None,
            };

            let data = match data {
                Some(data) => data,
                None => {
                    let span = proc_macro2::Span::call_site();
                    let msg = "not a struct with named fields";
                    return Err(syn::Error::new(span, msg));
                },
            };

            let span = proc_macro2::Span::call_site();
            let info = crate::parse::rules::Info {
                data: input.attrs,
                span,
            };

            let attr = [<$table Attr>]::try_from(info)?;
            let fields = data.fields.into_iter().map($field::try_from);
            let fields = fields.collect::<syn::Result<Vec<$field>>>()?;

            Ok(Self {
                attr,
                fields,
                generic: input.generics,
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
            let info = crate::parse::rules::Info {
                data: field.attrs,
                span,
            };
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
($vis:vis $i:ident { $((($n:ident)$r:tt $($t:tt)*),)* }) => {
paste::paste! {

    #[allow(non_camel_case_types)]
    enum [<$i Enum>] {
        $($n(typed!(! $($t)*)),)*
    }

    $vis struct $i {
        $(pub $n: typed!($r $($t)*),)*
    }

    impl syn::parse::Parse for [<$i Enum>] {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let span = input.span();
            match &*input.parse::<syn::Ident>()?.to_string() {
                $(stringify!($n) => token!(input, $($t)*).map(|v| {
                    let info = crate::parse::rules::Info::new(v, span);
                    let name = crate::parse::rules::Name::new(info, stringify!($n));
                    [<$i Enum>]::$n(name)
                }),)*
                n => Err(syn::Error::new(span, format!(
                    "unknown attribute: #[sqly({})]", n
                ))),
            }
        }
    }

    impl TryFrom<crate::parse::rules::Info<Vec<syn::Attribute>>> for $i {
        type Error = syn::Error;

        fn try_from(info: crate::parse::rules::Info<Vec<syn::Attribute>>) -> syn::Result<Self> {
            let iter = info.data.into_iter().filter(|a| a.path().is_ident("sqly")).map(|a| {
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
                                    "duplicate attribute: #[sqly({})]", stringify!($n)
                                )));
                            } },
                            ? => ~ !,
                            + => r#match!({ $($t)* } {
                                ! => $n.push(v),
                                ? => ~ !,
                                + => {
                                    let first: Option<&crate::parse::rules::Name<Vec<_>>> = $n.first();
                                    if first.map_or(false, |w| w.data.is_empty() || v.data.is_empty()) {
                                        return Err(syn::Error::new(v.span, format!(
                                            "duplicate attribute: #[sqly({})]", stringify!($n)
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
                    return Err(syn::Error::new(info.span, format!(
                        "missing attribute: #[sqly({})]", stringify!($n)
                    )));
                } },
                ? => $n,
                + => ~ !,
                * => $n,
            });)*

            Ok($i { $($n,)* })
        }

    }

} } }



macro_rules! vars {
{ $($vis:vis $e:ident$(: $a:ident)? { $(($($t:tt)*),)* })* } => {
    $(vari!{ $vis $e$(: $a)? { $(($($t)*),)* } })*
} }

macro_rules! vari {
{ $vis:vis $e:ident$(: $a:ident)? { $(($v:ident = $n:literal),)* } } => {
    vari!{ impl $vis $e $($a)? { $(($v, stringify!($n), $n),)* } true }
};
{ $vis:vis $e:ident$(: $a:ident)? { $(($v:ident = $n:ident),)* } } => {
    vari!{ impl $vis $e $($a)? { $(($v, stringify!($n), stringify!($n)),)* } false }
};
{ impl $vis:vis $e:ident $($a:ident)? { $(($v:ident, $n:expr, $s:expr),)* } $lit:tt } => {

    #[derive(Clone,Copy,PartialEq,Eq)]
    $vis enum $e {
        $($v,)*
    }

    r#some!({ $($a)? } {
        impl From<$e> for $($a)? {
            fn from(e: $e) -> Self {
                match e {
                    $($e::$v => Self::$v,)*
                }
            }
        }
        impl TryFrom<$($a)?> for $e {
            type Error = $($a)?;
            #[allow(unreachable_patterns)]
            fn try_from(a: $($a)?) -> std::result::Result<Self, $($a)?> {
                match a {
                    $(Self::Error::$v => Ok(Self::$v),)*
                    _ => Err(a),
                }
            }
        }
    });

    impl syn::parse::Parse for $e {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let span = input.span();
            let input = r#bool!({ $lit } {
                false => input.parse::<syn::Ident>()?.to_string(),
                true => input.parse().map(|s: syn::LitStr| s.value())?,
            });
            match input.as_str() {
                $($s => Ok($e::$v),)*
                n => {
                    let list = [$($n,)*].join(", ");
                    Err(syn::Error::new(span, format!(
                        r#bool!({ $lit } {
                            false => "unknown variant: {}\n must be one of: {}",
                            true => "unknown variant: \"{}\"\n must be one of: {}",
                        }), n, list
                    )))
                },
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

} }



macro_rules! token {
    ($i:expr, (= $($t:tt)*)*) => ({
        match $i.peek(syn::Token![=]) {
            true => token!($i, (= $($t)*)+),
            false => Ok(Vec::new()),
        }
    });
    ($i:expr, (= $($t:tt)*)+) => ({
        $i.parse::<syn::Token![=]>()?;
        let mut vec = Vec::new();
        vec.push(token!($i, $($t)*)?);
        while $i.peek(syn::Token![,]) {
            if $i.peek3(syn::Token![=]) {
                break;
            }
            $i.parse::<syn::Token![,]>()?;
            if $i.cursor().eof() {
                break;
            }
            vec.push(token!($i, $($t)*)?);
        }
        Ok(vec)
    });
    ($i:expr, (= $($t:tt)*)?) => ({
        match $i.peek(syn::Token![=]) {
            true => token!($i, (= $($t)*)!).map(Some),
            false => Ok(None),
        }
    });
    ($i:expr, (= $($t:tt)*)!) => ({
        $i.parse::<syn::Token![=]>()?;
        token!($i, $($t)*)
    });
    ($i:expr, $($t:tt)*) => ({
        let span = $i.span();
        token!(, $i, $($t)*).map(|v| {
            crate::parse::rules::Info::new(v, span)
        })
    });
    (, $i:expr, f64) => ( $i.parse().and_then(|s: syn::LitFloat| s.base10_parse()) );
    (, $i:expr, u64) => ( $i.parse().and_then(|s: syn::LitInt| s.base10_parse()) );
    (, $i:expr, i64) => ( $i.parse().and_then(|s: syn::LitInt| s.base10_parse()) );
    (, $i:expr, String) => ( $i.parse().map(|s: syn::LitStr| s.value()) );
    (, $i:expr, bool) => ( $i.parse().map(|s: syn::LitBool| s.value()) );
    (, $i:expr, $t:ty) => ( $i.parse::<$t>() );
    (, $i:expr,) => ( Ok(crate::parse::rules::Empty) );
}

macro_rules! typed {
    (?) => ( Option<crate::parse::rules::Name<crate::parse::rules::Info<crate::parse::rules::Empty>>> );
    (*) => ( Vec<crate::parse::rules::Name<crate::parse::rules::Info<crate::parse::rules::Empty>>> );
    (+) => ( Vec<crate::parse::rules::Name<crate::parse::rules::Info<crate::parse::rules::Empty>>> );
    (!) => ( crate::parse::rules::Name<crate::parse::rules::Info<crate::parse::rules::Empty>> );
    (? (= $t:ty)?) => ( Option<crate::parse::rules::Name<Option<crate::parse::rules::Info<$t>>>> );
    (? (= $t:ty)*) => ( Option<crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>>> );
    (? (= $t:ty)+) => ( Option<crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>>> );
    (? (= $t:ty)!) => ( Option<crate::parse::rules::Name<crate::parse::rules::Info<$t>>> );
    (* (= $t:ty)?) => ( Vec<crate::parse::rules::Name<Option<crate::parse::rules::Info<$t>>>> );
    (* (= $t:ty)*) => ( Vec<crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>>> );
    (* (= $t:ty)+) => ( Vec<crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>>> );
    (* (= $t:ty)!) => ( Vec<crate::parse::rules::Name<crate::parse::rules::Info<$t>>> );
    (+ (= $t:ty)?) => ( Vec<crate::parse::rules::Name<Option<crate::parse::rules::Info<$t>>>> );
    (+ (= $t:ty)*) => ( Vec<crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>>> );
    (+ (= $t:ty)+) => ( Vec<crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>>> );
    (+ (= $t:ty)!) => ( Vec<crate::parse::rules::Name<crate::parse::rules::Info<$t>>> );
    (! (= $t:ty)?) => ( crate::parse::rules::Name<Option<crate::parse::rules::Info<$t>>> );
    (! (= $t:ty)*) => ( crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>> );
    (! (= $t:ty)+) => ( crate::parse::rules::Name<Vec<crate::parse::rules::Info<$t>>> );
    (! (= $t:ty)!) => ( crate::parse::rules::Name<crate::parse::rules::Info<$t>> );
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

macro_rules! r#some {
    ({ $($_:tt)+ } { $($o:tt)* }) => { $($o)* };
    ({} { $($o:tt)* }) => {};
}
