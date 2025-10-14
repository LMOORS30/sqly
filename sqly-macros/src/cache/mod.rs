use std::collections::{
    btree_map as btree,
    BTreeMap as BTree,
};

use proc_macro2::TokenStream;

use std::sync::RwLockWriteGuard;
use std::sync::RwLockReadGuard;
use std::sync::OnceLock;
use std::sync::RwLock;

use crate::parse::*;

pub use construct::*;
pub use params::*;

mod construct;
mod params;



macro_rules! cache {
{ $($lower:ident: $upper:ident($table:ident),)* } => {

pub trait Cache {
    fn id(&self) -> Result<Id>;
    fn dep(&self) -> Result<Dep>;
    fn call(self) -> Result<TokenStream>;
}



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    ident: String,
}

impl TryFrom<&syn::Ident> for Id {
    type Error = syn::Error;
    fn try_from(ident: &syn::Ident) -> Result<Id> {
        Ok(Id { ident: ident.unraw() })
    }
}

impl TryFrom<&syn::Path> for Id {
    type Error = syn::Error;
    fn try_from(path: &syn::Path) -> Result<Id> {
        match path.segments.last() {
            None => {
                let msg = "invalid path: no segments\n\
                    note: required by sqly internals";
                return Err(syn::Error::new_spanned(path, msg));
            }
            Some(segment) => {
                if !segment.arguments.is_none() {
                    let msg = "invalid path: generics not supported\n\
                        note: required by sqly internals";
                    return Err(syn::Error::new_spanned(path, msg));
                }
                if path.is_ident("Self") {
                    let msg = "invalid path: Self\n\
                        note: enforced by sqly internals";
                    return Err(syn::Error::new_spanned(path, msg));
                }
                Id::try_from(&segment.ident)
            }
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ident)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key<T> {
    $($upper(T),)*
}

impl<T> Key<T> {
    fn into_inner(self) -> T {
        match self {
            $(Key::$upper(val) => val,)*
        }
    }
}

impl TryFrom<Key<&syn::Path>> for Key<Id> {
    type Error = syn::Error;
    fn try_from(key: Key<&syn::Path>) -> Result<Key<Id>> {
        Ok(match key {
            $(Key::$upper(id) => Key::$upper(id.try_into()?),)*
        })
    }
}

impl<T: std::borrow::Borrow<Id>> std::fmt::Display for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            $(Key::$upper(id) => {
                write!(f, "#[derive({})] {}", stringify!($upper), id.borrow())
            },)*
        }
    }
}



#[derive(Clone, Copy, PartialEq, Eq)]
enum Res {
    End,
    Art,
}

#[derive(Clone)]
pub struct Dep<'c> {
    src: Vec<(Key<&'c syn::Path>, Res)>,
}

impl<'c> Dep<'c> {

    pub fn new() -> Dep<'c> {
        Dep { src: Vec::new() }
    }

    pub fn art(&mut self, key: Key<&'c syn::Path>) -> &mut Dep<'c> {
        self.src.push((key, Res::Art));
        self
    }

    pub fn end(&mut self, key: Key<&'c syn::Path>) -> &mut Dep<'c> {
        self.src.push((key, Res::End));
        self
    }

}



struct Src<T> {
    dep: BTree<Key<Id>, Res>,
    val: T,
}

impl<T> Src<T> {

    fn res(&self) -> Res {
        match self.dep.len() {
            0 => Res::End,
            _ => Res::Art,
        }
    }

    fn pop(&mut self, key: &Key<Id>, res: Res) -> Option<Res> {
        let pop = match res {
            Res::End => !self.dep.is_empty(),
            Res::Art => match self.dep.get(&key) {
                Some(&res) => res == Res::Art,
                None => false,
            }
        };
        match pop {
            false => None,
            true => match self.dep.remove(&key) {
                Some(_) => Some(self.res()),
                None => None,
            }
        }
    }

}



#[derive(Default)]
pub struct Local {
    $($lower: BTree<Id, Option<$table>>,)*
}

#[allow(dead_code)]
impl Local {
paste::paste! {

    $(pub fn [<has_ $lower>](&self, id: &Id) -> bool {
        self.$lower.get(id).is_some()
    })*

    $(pub fn [<get_ $lower>](&self, id: &Id) -> Result<&$table> {
        match self.$lower.get(id).and_then(Option::as_ref) {
            Some(entry) => Ok(entry),
            None => {
                let key = Key::$upper(id);
                let msg = format!("missing definition: {key}\n\
                    note: this error should not occur");
                Err(syn::Error::new(Span::call_site(), msg))
            }
        }
    })*

    $(pub fn [<pop_ $lower>](&mut self, id: &Id) -> Result<Option<$table>> {
        match self.$lower.get_mut(id) {
            Some(entry) => Ok(entry.take()),
            None => {
                let key = Key::$upper(id);
                let msg = format!("missing definition: {key}\n\
                    note: this error should not occur");
                Err(syn::Error::new(Span::call_site(), msg))
            }
        }
    })*

    $(pub fn [<put_ $lower>](&mut self, id: Id, table: $table) -> Result<()> {
        self.$lower.insert(id, Some(table));
        Ok(())
    })*

} }



pub struct Guard<T>(T);

pub type ReadGuard = Guard<RwLockReadGuard<'static, Store>>;
pub type WriteGuard = Guard<RwLockWriteGuard<'static, Store>>;

#[derive(Default)]
pub struct Store {
    $($lower: BTree<Id, Src<<$table as Safe>::Safe>>,)*
}

static STORE: OnceLock<RwLock<Store>> = OnceLock::new();

pub mod cache {
    use super::*;

    pub fn fetch() -> ReadGuard {
        Guard(STORE.get_or_init(Default::default).read().unwrap())
    }

    pub fn store() -> WriteGuard {
        Guard(STORE.get_or_init(Default::default).write().unwrap())
    }
}

#[allow(dead_code)]
impl ReadGuard {

    $(pub fn $lower(&self, id: &Id) -> Result<&<$table as Safe>::Safe> {
        match self.0.$lower.get(id) {
            Some(tree) => Ok(&tree.val),
            None => {
                let key = Key::$upper(id);
                let msg = format!("missing definition: {key}\n\
                    note: this error should not occur");
                Err(syn::Error::new(Span::call_site(), msg))
            }
        }
    })*

    fn call(&self, key: &Key<Id>) -> Result<TokenStream> {
        match key {
            $(Key::$upper(id) => match self.0.$lower.get(id) {
                Some(tree) => tree.val.sync()?.call(),
                None => {
                    let key = Key::$upper(id);
                    let msg = format!("missing definition: {key}\n\
                        note: this error should not occur");
                    Err(syn::Error::new(Span::call_site(), msg))
                }
            },)*
        }
    }

}

impl WriteGuard {

    fn pop(&mut self, key: &Key<Id>, res: Res) -> Vec<Key<Id>> {
        let mut vec = Vec::new();
        $(vec.extend(self.0.$lower.iter_mut().filter_map(|(id, val)| {
            val.pop(key, res).and_then(|pop| match pop {
                Res::End => Some(Key::$upper(id.clone())),
                Res::Art => None,
            })
        }));)*
        vec
    }

    fn put(&mut self, key: &Key<Id>, res: Res) -> Vec<Key<Id>> {
        let mut new = self.pop(key, res);
        let mut all = Vec::new();
        while !new.is_empty() {
            let add = new.iter().flat_map(|key| {
                self.pop(key, Res::End)
            }).collect();
            all.append(&mut new);
            new = add;
        }
        all
    }

}



impl WriteGuard {

    fn dep(&self, key: &Key<Id>) -> Option<&BTree<Key<Id>, Res>> {
        match key {
            $(Key::$upper(id) => match self.0.$lower.get(id) {
                Some(src) => Some(&src.dep),
                None => None,
            },)*
        }
    }

    fn check(&self, key: &Key<Id>, path: Key<&syn::Path>) -> Option<TokenStream> {
        self.dep(&key).is_none().then(|| match path {
            $(Key::$upper(path) => quote::quote! { $lower::<#path> },)*
        })
    }

    $(pub fn $lower(mut self, val: $table) -> Result<TokenStream>
    where $table: Cache + Safe<Error = syn::Error>
    {
        let id = val.id()?;
        let dep = val.dep()?;
        let mut src = Src {
            val: val.send()?,
            dep: BTree::new(),
        };
        let mut out = TokenStream::new();

        for (path, res) in dep.src {
            let key = path.try_into()?;
            if let Some(check) = self.check(&key, path) {
                let krate = val.krate()?;
                let typed = quote::quote! {
                    const _: () = #krate::require::#check();
                };
                out.extend(typed);
            }
            if match (self.dep(&key), res) {
                (Some(dep), Res::End) => !dep.is_empty(),
                (Some(_), Res::Art) => false,
                (None, _) => true,
            } {
                src.dep.insert(key, res);
            }
        }

        let res = src.res();
        let key = Key::$upper(id);
        let vec = self.put(&key, res);
        let id = key.into_inner();

        match self.0.$lower.entry(id) {
            btree::Entry::Occupied(entry) => {
                let key = Key::$upper(entry.key());
                let msg = format!("duplicate definition: {key}\n\
                    note: cannot #[derive({})] on multiple structs with the same identifier",
                    stringify!($upper));
                return Err(syn::Error::new(Span::call_site(), msg));
            }
            btree::Entry::Vacant(entry) => {
                entry.insert(src);
            }
        }

        drop(self);
        let read = cache::fetch();
        if let Res::End = res {
            let put = val.call()?;
            out.extend(put.into_iter())
        }
        for put in vec {
            let put = read.call(&put)?;
            out.extend(put.into_iter())
        }
        Ok(out)
    })*

}

} }



cache! {
    table: Table(QueryTable),
    delete: Delete(DeleteTable),
    insert: Insert(InsertTable),
    select: Select(SelectTable),
    update: Update(UpdateTable),
}
