use std::collections::{HashMap as Map};
use std::fmt::{self, Write, Display};
use std::borrow::Borrow;

pub use std::cell::RefCell;
pub use std::hash::Hash;
pub use std::rc::Rc;

use crate::parse::*;



pub enum Index<T> {
    Value(usize),
    Unset(T),
}

impl<T> Placer for Index<T> {
    type State = Vec<T>;
    type Item<'c> = usize
        where Self: 'c;
    fn place(&mut self, state: &mut Self::State) -> Result<Self::Item<'_>> {
        Ok(match self {
            Index::Value(i) => *i,
            Index::Unset(_) => {
                let i = state.len() + 1;
                let val = Index::Value(i);
                match core::mem::replace(self, val) {
                    Index::Unset(ident) => state.push(ident),
                    Index::Value(_) => unreachable!(),
                };
                i
            }
        })
    }
}



pub struct Dollar<T>(pub T);

impl<T: Placer> Placer for Dollar<T> {
    type State = T::State;
    type Item<'c> = Dollar<T::Item<'c>>
        where Self: 'c;
    fn place(&mut self, state: &mut Self::State) -> Result<Self::Item<'_>> {
        Ok(Dollar(self.0.place(state)?))
    }
}

impl<T: Display> Display for Dollar<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "$")?;
        write!(f, "{}", self.0)?;
        Ok(())
    }
}



impl Displacer for usize {}
impl Displacer for String {}
impl<'c> Displacer for &'c str {}

pub trait Displacer: Display {}

pub trait Placer {
    type State;
    type Item<'c>: Display
        where Self: 'c;
    fn place(&mut self, state: &mut Self::State) -> Result<Self::Item<'_>>;
}

impl<T: Displacer> Placer for T {
    type State = ();
    type Item<'c> = &'c Self
        where Self: 'c;
    fn place(&mut self, _: &mut Self::State) -> Result<Self::Item<'_>> {
        Ok(self)
    }
}

impl<T: Placer> Placer for Rc<RefCell<T>> {
    type State = T::State;
    type Item<'c> = String
        where Self: 'c;
    fn place(&mut self, state: &mut Self::State) -> Result<Self::Item<'_>> {
        Ok(self.as_ref().borrow_mut().place(state)?.to_string())
    }
}



pub struct Params<K, V: Placer> {
    pub state: V::State,
    map: Map<K, V>,
}

impl<K, V: Placer> Params<K, V> {

    pub fn state(state: V::State) -> Self {
        Self {
            state,
            map: Default::default(),
        }
    }

}

impl<K, V: Placer> Default for Params<K, V>
where V::State: Default {

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

    pub fn place<'c>(&mut self, val: &'c mut V) -> Result<V::Item<'c>> {
        val.place(&mut self.state)
    }

}

impl<V: Placer> Params<String, V> {

    pub fn ensure<K: AsRef<str>>(&mut self, key: K) {
        if let Some(res) = self.map.remove(key.as_ref()) {
            self.emplace(format!("self__{}", key.as_ref()), res);
        }
    }

    pub fn emplace<K: Display>(&mut self, key: K, val: V) {
        if let Some(res) = self.map.insert(key.to_string(), val) {
            self.emplace(format!("self__{key}"), res);
        }
    }

}

impl<K: Borrow<str> + Hash + Eq, V: Placer> Params<K, V> {

    pub fn output(&mut self, input: &[&Info<String>]) -> Result<String> {
        let output = input.iter().map(|select| {
            let line = select.data.trim_ascii();
            self.apply(line, select.span)
        }).collect::<Result<Vec<_>>>()?;
        Ok(output.join("\n"))
    }

    pub fn apply(&mut self, src: &str, span: proc_macro2::Span) -> Result<String> {
        let mut dst = String::new();
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
                        '{' => format!("invalid identifier: \"{var}\"\n\
                            help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                        _ => format!("invalid identifier: \"{var}\"\n\
                            help: use \"$${var}\" to escape and resolve to the literal \"${var}\""),
                    };
                    return Err(syn::Error::new(span, msg));
                }
            };

            match self.map.get_mut(&ident) {
                Some(val) => {
                    let res = val.place(&mut self.state)?;
                    write!(&mut dst, "{}", res).unwrap()
                }
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
        Ok(dst)
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
            let span = proc_macro2::Span::call_site();
            params.apply(src, span)
        }

        fn replaced(src: &str) -> String {
            result(src).unwrap()
        }

        fn errored(err: &str, src: &str) {
            assert!(result(src).unwrap_err().to_string().contains(err));
        }

        #[test]
        fn empty() {
            assert_eq!(replaced(""), "");
        }

        #[test]
        fn copy() {
            assert_eq!(replaced("copy"), "copy");
        }

        #[test]
        fn replace() {
            assert_eq!(replaced("$one"), "1");
            assert_eq!(replaced("$two $one"), "2 1");
            assert_eq!(replaced("$two${ one }$two"), "212");
            assert_eq!(replaced("${one}$two${one}"), "121");
            assert_eq!(replaced("r#${r#table}#"), "r#elbat#");
            assert_eq!(replaced("{$r#other#}"), "{rehto#}");
            assert_eq!(replaced("{${SELF}}"), "{self}");
            assert_eq!(replaced("${ r#mod }"), "mod");
            assert_eq!(replaced("$r#mod"), "mod");
        }

        #[test]
        fn escape() {
            assert_eq!(replaced("$$"), "$");
            assert_eq!(replaced("$$$$r#"), "$$r#");
            assert_eq!(replaced("$$table"), "$table");
            assert_eq!(replaced("$$$table$$"), "$elbat$");
            assert_eq!(replaced("$one$$one$one"), "1$one1");
            assert_eq!(replaced("$${ table }"), "${ table }");
            assert_eq!(replaced("$${ $table }"), "${ elbat }");
            assert_eq!(replaced("$${ r#$$ "), "${ r#$ ");
            assert_eq!(replaced("$${"), "${");
        }

        #[test]
        fn statement() {
            assert_eq!(replaced(
                "$INNER JOIN other AS $other ON $other.id = $table.other_id"
            ), "INNER JOIN other AS rehto ON rehto.id = elbat.other_id");
            assert_eq!(replaced(
                r#"$inner JOIN other AS "${other}" ON $other.id="$table".other_id"#
            ), r#"LEFT JOIN other AS "rehto" ON rehto.id="elbat".other_id"#);
            assert_eq!(replaced(
                "$inner JOIN other_a AS ${other}_a ON ${other}_a.id = $table.other_a_id\n\
                $inner JOIN other_b AS ${other}_b ON ${other}_b.id = $table.other_b_id\n\
                $INNER JOIN other AS $other ON\n\
                    $other.id_a = ${other}_a.id OR\n\
                    $other.id_b = ${other}_b.id"
            ), "LEFT JOIN other_a AS rehto_a ON rehto_a.id = elbat.other_a_id\n\
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
