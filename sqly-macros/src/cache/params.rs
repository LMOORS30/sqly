use std::collections::{BTreeMap as BTree};
use std::collections::{HashMap as Map};
use std::fmt::{Write, Display};

pub use std::borrow::Borrow;
pub use std::cell::RefCell;
pub use std::hash::Hash;
pub use std::rc::Rc;

use crate::parse::*;



pub struct Index<T> {
    index: Option<usize>,
    value: T,
}

impl<T> Index<T> {
    pub fn unset(value: T) -> Self {
        Self {
            index: None,
            value,
        }
    }
}

impl<'c, T: Field> Placer<&'c T> for Index<&'c T> {
    type State = Vec<&'c T>;
    fn place<W: Wrapper<&'c T>>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        if dst.outer() {
            dst.state(self.value);
            let dst = dst.inner();
            dst.push_str("{");
            dst.push_arg(self.value.ident().unraw());
            dst.push_str("}");
            Ok(())
        } else {
            let i = self.index.get_or_insert_with(|| {
                state.push(self.value);
                state.len()
            });
            dst.push_arg(i);
            Ok(())
        }
    }
}



pub struct Dollar<V>(pub V);

impl<V: Placer<T>, T> Placer<T> for Dollar<V> {
    type State = V::State;
    fn place<W: Wrapper<T>>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        dst.push_str("$");
        self.0.place(state, dst)?;
        Ok(())
    }
}



pub struct Format<'c, T> {
    escape: Escape<String>,
    state: BTree<&'c syn::Ident, &'c T>,
}

impl<'c, T> Default for Format<'c, T> {
    fn default() -> Self {
        Self {
            escape: Escape(String::new()),
            state: BTree::new(),
        }
    }
}

impl<'c, T> Format<'c, T> {
    pub fn into_inner(self) -> (String, BTree<&'c syn::Ident, &'c T>) {
        (self.escape.into_inner(), self.state)
    }
}

impl<'c, T> Write for Format<'c, T> {
    fn write_str(&mut self, str: &str) -> std::fmt::Result {
        self.escape.write_str(str)
    }
}

impl<'c, T: Field> Wrapper<&'c T> for Format<'c, T> {
    type Inner = String;
    fn inner(&mut self) -> &mut Self::Inner { &mut self.escape.0 }
    fn outer(&self) -> bool { true }
    fn state(&mut self, value: &'c T) {
        self.state.insert(value.ident(), value);
    }
}



impl Displacer for str {}
impl Displacer for usize {}
impl Displacer for String {}
impl Displacer for Cow<'_, str> {}
impl<T: ?Sized + Displacer> Displacer for &T {}

pub trait Displacer: Display {}

pub trait Placer<T> {
    type State: Default;
    fn place<W: Wrapper<T>>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()>;
}

impl<V: Displacer, T> Placer<T> for V {
    type State = ();
    fn place<W: Wrapper<T>>(&mut self, _: &mut Self::State, dst: &mut W) -> Result<()> {
        dst.push_arg(self);
        Ok(())
    }
}

impl<V: Placer<T>, T> Placer<T> for Rc<RefCell<V>> {
    type State = V::State;
    fn place<W: Wrapper<T>>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        self.as_ref().borrow_mut().place(state, dst)
    }
}

impl<L: Placer<T>, R: Placer<T>, T> Placer<T> for Either<L, R> {
    type State = (L::State, R::State);
    fn place<W: Wrapper<T>>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        match self {
            Left(left) => left.place(&mut state.0, dst),
            Right(right) => right.place(&mut state.1, dst),
        }
    }
}



pub struct Drain;

pub struct Escape<W>(W);

impl Unwrapper for Drain {}
impl Unwrapper for String {}

pub trait Unwrapper: Write {}

pub trait Wrapper<V>: Write {
    type Inner: Wrapper<V>;
    fn inner(&mut self) -> &mut Self::Inner;
    fn state(&mut self, value: V);
    fn outer(&self) -> bool;
}

impl<W: Unwrapper, T> Wrapper<T> for W {
    type Inner = Self;
    fn inner(&mut self) -> &mut Self::Inner { self }
    fn outer(&self) -> bool { false }
    fn state(&mut self, _: T) {}
}

impl<W: Wrapper<T>, T> Wrapper<T> for Escape<W> {
    type Inner = W;
    fn inner(&mut self) -> &mut Self::Inner { &mut self.0 }
    fn outer(&self) -> bool { self.0.outer() }
    fn state(&mut self, value: T) {
        self.0.state(value);
    }
}

impl Write for Drain {
    fn write_str(&mut self, _: &str) -> std::fmt::Result { Ok(()) }
}

impl<W: Write> Write for Escape<W> {
    fn write_str(&mut self, mut str: &str) -> std::fmt::Result {
        while let Some(i) = str.find(['{', '}']) {
            self.0.write_str(&str[..=i])?;
            self.0.write_str(&str[i..=i])?;
            str = &str[i + 1..];
        }
        self.0.write_str(str)?;
        Ok(())
    }
}

impl<W> Escape<W> {
    pub fn into_inner(self) -> W { self.0 }
}

pub trait Push: Write {
    fn push_str(&mut self, s: &str) {
        self.write_str(s).unwrap()
    }
    fn push_arg<T: Display>(&mut self, val: T) {
        write!(self, "{}", val).unwrap()
    }
}

impl<W: Write> Push for W {}



pub struct Params<K, V: Placer<T>, T = ()> {
    pub state: V::State,
    pub map: Map<K, V>,
}

impl<K, V: Placer<T>, T> Params<K, V, T> {

    pub fn state(state: V::State) -> Self {
        Self {
            state,
            map: Default::default(),
        }
    }

}

impl<K, V: Placer<T>, T> Default for Params<K, V, T> {

    fn default() -> Self {
        Self {
            state: Default::default(),
            map: Default::default(),
        }
    }

}

impl<K: Hash + Eq, V: Placer<T>, T> Params<K, V, T> {

    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.get(key)
    }

    pub fn put<Q: Into<K>>(&mut self, key: Q, val: V) -> Option<V> {
        self.map.insert(key.into(), val)
    }

}

impl<V: Placer<T>, T> Params<String, V, T> {

    pub fn ensure<K: AsRef<str>>(&mut self, key: K) {
        if let Some(res) = self.map.remove(key.as_ref()) {
            self.displace(format!("self__{}", key.as_ref()), res);
        }
    }

    pub fn displace<K: Display>(&mut self, key: K, val: V) {
        if let Some(res) = self.map.insert(key.to_string(), val) {
            self.displace(format!("self__{key}"), res);
        }
    }

}

impl<K, V: Placer<T>, T> Params<K, V, T> {

    pub fn place<W: Wrapper<T>>(&mut self, dst: &mut W, val: &mut V) -> Result<()> {
        val.place(&mut self.state, dst)
    }

    pub fn drain<'c, I, J: 'c>(&mut self, i: I) -> Result<()>
    where
        V: 'c,
        I: IntoIterator<Item = &'c mut (J, V)>,
    {
        let drain = &mut Drain;
        for (_, val) in i.into_iter() {
            val.place(&mut self.state, drain)?;
        }
        Ok(())
    }

    pub fn take(&mut self) -> V::State {
        std::mem::take(&mut self.state)
    }

}

impl<K: Borrow<str> + Hash + Eq, V: Placer<T>, T> Params<K, V, T> {

    pub fn output<W: Wrapper<T>>(&mut self, dst: &mut W, input: &[&Info<String>]) -> Result<()> {
        let mut first = true;
        for info in input {
            if !first { dst.push_str("\n\t"); }
            let line = info.data.trim_matches(|c: char| {
                c.is_ascii_whitespace()
            });
            self.apply(dst, line, info.span())?;
            first = false;
        }
        Ok(())
    }

    pub fn apply<W: Wrapper<T>>(&mut self, dst: &mut W, src: &str, span: Span) -> Result<()> {
        let mut src = src;

        while let Some(i) = src.find('$') {
            let mut chars = src[i..].chars();
            let next = chars.nth(1);

            if next == Some('$') {
                dst.push_str(&src[..=i]);
                src = &src[i + 2..];
                continue;
            }

            let var = match next {
                Some('{') => {
                    let j = match src[i + 2..].find('}') {
                        Some(j) => j + i + 2,
                        None => {
                            let msg = "unmatched opening brace: \"${\" expects a closing \"}\"\n\
                                help: use \"$${\" to escape and resolve to the literal \"${\"";
                            return Err(syn::Error::new(span, msg));
                        }
                    };
                    let var = &src[i + 2..j];
                    dst.push_str(&src[..i]);
                    src = &src[j + 1..];
                    var
                }
                Some(char) => {
                    let o = if char == 'r' && chars.next() == Some('#') { 3 } else { 1 };
                    let j = src[i + o..].find(|c| !unicode_ident::is_xid_continue(c));
                    let j = j.map_or(src.len(), |j| j + i + o);
                    let var = &src[i + 1..j];
                    dst.push_str(&src[..i]);
                    src = &src[j..];
                    var
                }
                None => {
                    let var = &src[i + 1..];
                    src = &src[i + 1..];
                    var
                }
            };

            if var.chars().all(|c| c.is_whitespace()) {
                let msg = match next {
                    Some('{') => format!("missing identifier: \"${{{var}}}\" is expected to enclose an identifier\n\
                        help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                    Some(char) => format!("missing identifier: \"$\" is expected to precede an identifier\n\
                        help: use \"$${char}\" to escape and resolve to the literal \"${char}\""),
                    None => format!("missing identifier: \"$\" is expected to precede an identifier\n\
                        help: use \"$$\" to escape and resolve to the literal \"$\""),
                };
                return Err(syn::Error::new(span, msg));
            }

            let ident = match syn::parse_str::<syn::Ident>(var) {
                Ok(ident) => ident.unraw(),
                Err(_) => {
                    let msg = match next.unwrap_or('\0') {
                        '{' => format!("invalid identifier: \"${{{var}}}\" is expected to be an identifier\n\
                            help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                        _ => format!("invalid identifier: \"${var}\" is expected to be an identifier\n\
                            help: use \"$${var}\" to escape and resolve to the literal \"${var}\""),
                    };
                    return Err(syn::Error::new(span, msg));
                }
            };

            match self.map.get_mut(&ident) {
                Some(val) => val.place(&mut self.state, dst)?,
                None => {
                    let mut params = self.map.keys().map(|key| key.borrow()).collect::<Vec<_>>();
                    params.sort_unstable_by_key(|&params| (params.split("__").count(), params));
                    let params = params.join(", ");
                    let msg = match next.unwrap_or('\0') {
                        '{' => format!("unknown parameter: {var}\
                                      \n known parameters: {params}\n\
                            help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                        _ => format!("unknown parameter: {var}\
                                    \n known parameters: {params}\n\
                            help: use \"$${var}\" to escape and resolve to the literal \"${var}\""),
                    };
                    return Err(syn::Error::new(span, msg));
                }
            }
        }

        dst.push_str(src);
        Ok(())
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    mod replace {
        use super::*;

        fn result(src: &str) -> Result<String> {
            let mut params = Params::<&str, _>::default();
            params.put("one", "1");
            params.put("two", "2");
            params.put("mod", "mod");
            params.put("SELF", "self");
            params.put("inner", "LEFT");
            params.put("INNER", "INNER");
            params.put("table", "elbat");
            params.put("other", "rehto");
            let mut dst = String::new();
            let span = Span::call_site();
            params.apply(&mut dst, src, span)?;
            Ok(dst)
        }

        fn replaced(src: &str, dst: &str) {
            let res = result(src).map_err(|_| ());
            assert_eq!(res, Ok(dst.to_string()));
        }

        fn errored(err: &str, src: &str) {
            let res = result(src).map_err(|msg| msg.to_string()).map_err(|msg| {
                msg.find(err).map_or(msg, |_| err.to_string())
            });
            assert_eq!(res, Err(err.to_string()));
        }

        #[test]
        fn empty() {
            replaced("", "");
        }

        #[test]
        fn copy() {
            replaced("copy", "copy");
        }

        #[test]
        fn replace() {
            replaced("$one", "1");
            replaced("$two $one", "2 1");
            replaced("$two${ one }$two", "212");
            replaced("${one}$two${one}", "121");
            replaced("r#${r#table}#", "r#elbat#");
            replaced("{$r#other#}", "{rehto#}");
            replaced("{${SELF}}", "{self}");
            replaced("${ r#mod }", "mod");
            replaced("$r#mod", "mod");
        }

        #[test]
        fn escape() {
            replaced("$$", "$");
            replaced("$$$$r#", "$$r#");
            replaced("$$table", "$table");
            replaced("$$$table$$", "$elbat$");
            replaced("$one$$one$one", "1$one1");
            replaced("$${ table }", "${ table }");
            replaced("$${ $table }", "${ elbat }");
            replaced("$${ r#$$ ", "${ r#$ ");
            replaced("$${", "${");
        }

        #[test]
        fn statement() {
            replaced(
                "$INNER JOIN other AS $other ON $other.id = $table.other_id"
            , "INNER JOIN other AS rehto ON rehto.id = elbat.other_id");
            replaced(
                r#"$inner JOIN other AS "${other}" ON $other.id="$table".other_id"#
            , r#"LEFT JOIN other AS "rehto" ON rehto.id="elbat".other_id"#);
            replaced(
                "$inner JOIN other_a AS ${other}_a ON ${other}_a.id = $table.other_a_id\n\
                $inner JOIN other_b AS ${other}_b ON ${other}_b.id = $table.other_b_id\n\
                $INNER JOIN other AS $other ON\n\
                    $other.id_a = ${other}_a.id OR\n\
                    $other.id_b = ${other}_b.id"
            , "LEFT JOIN other_a AS rehto_a ON rehto_a.id = elbat.other_a_id\n\
                LEFT JOIN other_b AS rehto_b ON rehto_b.id = elbat.other_b_id\n\
                INNER JOIN other AS rehto ON\n\
                    rehto.id_a = rehto_a.id OR\n\
                    rehto.id_b = rehto_b.id"
            );
        }

        #[test]
        fn missing() {
            let err = "missing";
            errored(err, "$");
            errored(err, "$ $");
            errored(err, "$ {");
            errored(err, "$$$");
            errored(err, "$${$}");
            errored(err, "$one$");
            errored(err, "${   }");
            errored(err, "r#${}");
            errored(err, "$ r#");
        }

        #[test]
        fn unmatched() {
            let err = "unmatched";
            errored(err, "${");
            errored(err, "${$");
            errored(err, "${${");
            errored(err, "$$${");
            errored(err, "$${}${");
            errored(err, "$one${");
            errored(err, "${$one");
            errored(err, "${r#{r");
            errored(err, "${r#");
        }

        #[test]
        fn invalid() {
            let err = "invalid";
            errored(err, "$_");
            errored(err, "$1");
            errored(err, "$mod");
            errored(err, "$r#self");
            errored(err, "${ $one }");
            errored(err, "${two one}");
            errored(err, "$r#{r}");
            errored(err, "${r#}");
            errored(err, "$r#");
        }

        #[test]
        fn unknown() {
            let err = "unknown";
            errored(err, "$a");
            errored(err, "$a1");
            errored(err, "$_one");
            errored(err, "$two_");
            errored(err, "${ r#b }");
            errored(err, "${ r#_2 }");
            errored(err, "$Table");
            errored(err, "$elbat");
            errored(err, "${__}");
        }

    }

    mod escape {
        use super::*;

        fn escaped(src: &str, dst: &str) {
            let mut esc = Escape(String::new());
            esc.write_str(src).unwrap();
            let res = esc.into_inner();
            assert_eq!(res, dst);
        }

        #[test]
        fn empty() {
            escaped("", "");
        }

        #[test]
        fn copy() {
            escaped("copy", "copy");
        }

        #[test]
        fn left() {
            escaped("{", "{{");
            escaped("{{", "{{{{");
            escaped(" { ", " {{ ");
            escaped("{  {", "{{  {{");
            escaped("  { {{ ", "  {{ {{{{ ");
        }

        #[test]
        fn right() {
            escaped("}", "}}");
            escaped("}}", "}}}}");
            escaped(" } ", " }} ");
            escaped("}  }", "}}  }}");
            escaped("  } }} ", "  }} }}}} ");
        }

        #[test]
        fn both() {
            escaped("{}", "{{}}");
            escaped("}{", "}}{{");
            escaped(" { } ", " {{ }} ");
            escaped(" } { ", " }} {{ ");
            escaped("}{}}{{}{", "}}{{}}}}{{{{}}{{");
        }

    }

}
