use std::collections::{HashMap as Map};
use std::fmt::{Write, Display};
use std::borrow::Borrow;

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

impl<T: Copy> Placer for Index<T> {
    type State = Vec<T>;
    fn place<W: Write>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        let i = self.index.get_or_insert_with(|| {
            state.push(self.value);
            state.len()
        });
        dst.push_arg(i);
        Ok(())
    }
}



pub struct Dollar<T>(pub T);

impl<T: Placer> Placer for Dollar<T> {
    type State = T::State;
    fn place<W: Write>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        dst.push_str("$");
        self.0.place(state, dst)?;
        Ok(())
    }
}



impl Displacer for str {}
impl Displacer for usize {}
impl Displacer for String {}
impl<'c, T: ?Sized + Displacer> Displacer for &'c T {}

pub trait Displacer: Display {}

pub trait Placer {
    type State: Default;
    fn place<W: Write>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()>;
}

impl<T: Displacer> Placer for T {
    type State = ();
    fn place<W: Write>(&mut self, _: &mut Self::State, dst: &mut W) -> Result<()> {
        dst.push_arg(self);
        Ok(())
    }
}

impl<T: Placer> Placer for Rc<RefCell<T>> {
    type State = T::State;
    fn place<W: Write>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        self.as_ref().borrow_mut().place(state, dst)
    }
}

impl<L: Placer, R: Placer> Placer for Either<L, R> {
    type State = (L::State, R::State);
    fn place<W: Write>(&mut self, state: &mut Self::State, dst: &mut W) -> Result<()> {
        match self {
            Left(left) => left.place(&mut state.0, dst),
            Right(right) => right.place(&mut state.1, dst),
        }
    }
}



pub trait Push: Write {
    fn push_str(&mut self, s: &str) {
        self.write_str(s).unwrap()
    }
    fn push_arg<T: Display>(&mut self, val: T) {
        write!(self, "{}", val).unwrap()
    }
}

impl<T: Write> Push for T {}



pub struct Params<K, V: Placer> {
    pub state: V::State,
    pub map: Map<K, V>,
}

impl<K, V: Placer> Params<K, V> {

    pub fn state(state: V::State) -> Self {
        Self {
            state,
            map: Default::default(),
        }
    }

}

impl<K, V: Placer> Default for Params<K, V> {

    fn default() -> Self {
        Self {
            state: Default::default(),
            map: Default::default(),
        }
    }

}

impl<K: Hash + Eq, V: Placer> Params<K, V> {

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

impl<V: Placer> Params<String, V> {

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

impl<K, V: Placer> Params<K, V> {

    pub fn place<W: Write>(&mut self, dst: &mut W, val: &mut V) -> Result<()> {
        val.place(&mut self.state, dst)
    }

}

impl<K: Borrow<str> + Hash + Eq, V: Placer> Params<K, V> {

    pub fn output<W: Write>(&mut self, dst: &mut W, input: &[&Info<String>]) -> Result<()> {
        let mut first = true;
        for info in input {
            if !first { dst.push_str("\n"); }
            let line = info.data.trim_ascii();
            self.apply(dst, line, info.span)?;
            first = false;
        }
        Ok(())
    }

    pub fn apply<W: Write>(&mut self, dst: &mut W, src: &str, span: Span) -> Result<()> {
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
                        '{' => format!("unknown parameter: {var}\n \
                            known parameters: {params}\n\
                            help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                        _ => format!("unknown parameter: {var}\n \
                            known parameters: {params}\n\
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

}
