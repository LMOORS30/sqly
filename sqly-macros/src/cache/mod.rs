use std::collections::{
    btree_map as map,
    BTreeMap as Map,
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



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    ident: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    $($upper(Id),)*
}

impl TryFrom<&syn::Ident> for Id {
    type Error = syn::Error;
    fn try_from(ident: &syn::Ident) -> Result<Self> {
        Ok(Self { ident: ident.unraw() })
    }
}

impl TryFrom<&syn::Path> for Id {
    type Error = syn::Error;
    fn try_from(path: &syn::Path) -> Result<Self> {
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

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            $(Self::$upper(id) => {
                write!(f, "#[derive({})] {}", stringify!($upper), id)
            },)*
        }
    }
}



#[derive(Clone, PartialEq, Eq, Copy)]
enum Res {
    End,
    Art,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Dep {
    set: Map<Key, Res>,
}

struct Tree<T> {
    dep: Dep,
    val: T,
}

impl Dep {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn art(&mut self, key: Key) -> &mut Self {
        self.set.entry(key).or_insert(Res::Art);
        self
    }

    pub fn end(&mut self, key: Key) -> &mut Self {
        self.set.insert(key, Res::End);
        self
    }

    fn res(&self) -> Res {
        match self.set.len() {
            0 => Res::End,
            _ => Res::Art,
        }
    }

    fn pop(&mut self, key: &Key, res: Res) -> Option<Res> {
        let pop = match res {
            Res::End => !self.set.is_empty(),
            Res::Art => match self.set.get(&key) {
                Some(&res) => res == Res::Art,
                None => false,
            }
        };
        match pop {
            false => None,
            true => match self.set.remove(&key) {
                Some(_) => Some(self.res()),
                None => None,
            }
        }
    }

}



pub struct Guard<T>(T);

pub type ReadGuard = Guard<RwLockReadGuard<'static, Store>>;
pub type WriteGuard = Guard<RwLockWriteGuard<'static, Store>>;

#[derive(Default)]
pub struct Store {
    $($lower: Map<Id, Tree<<$table as Safe>::Safe>>,)*
}

#[derive(Default)]
pub struct Local {
    $($lower: Map<Id, Option<$table>>,)*
}

static STORE: OnceLock<RwLock<Store>> = OnceLock::new();

pub fn fetch() -> ReadGuard {
    Guard(STORE.get_or_init(Default::default).read().unwrap())
}

pub fn store() -> WriteGuard {
    Guard(STORE.get_or_init(Default::default).write().unwrap())
}

pub mod cache {
    pub use super::fetch;
    pub use super::store;
}

#[allow(dead_code)]
impl Local {
paste::paste! {

    $(pub fn [<has_ $lower>](&self, id: &Id) -> bool {
        self.$lower.get(id).and_then(Option::as_ref).is_some()
    })*

    $(pub fn [<get_ $lower>](&self, id: &Id) -> Result<&$table> {
        match self.$lower.get(id).and_then(Option::as_ref) {
            Some(entry) => Ok(entry),
            None => {
                let key = Key::$upper(id.clone());
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
                let key = Key::$upper(id.clone());
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

#[allow(dead_code)]
impl Guard<RwLockReadGuard<'static, Store>> {

    $(pub fn $lower(&self, id: &Id) -> Result<&<$table as Safe>::Safe> {
        match self.0.$lower.get(id) {
            Some(tree) => Ok(&tree.val),
            None => {
                let key = Key::$upper(id.clone());
                let msg = format!("missing definition: {key}\n\
                    note: this error should not occur");
                Err(syn::Error::new(Span::call_site(), msg))
            }
        }
    })*

    fn call(&self, key: &Key) -> Result<TokenStream> {
        match key {
            $(Key::$upper(id) => match self.0.$lower.get(id) {
                Some(tree) => tree.val.sync()?.call(),
                None => {
                    let key = Key::$upper(id.clone());
                    let msg = format!("missing definition: {key}\n\
                        note: this error should not occur");
                    Err(syn::Error::new(Span::call_site(), msg))
                }
            },)*
        }
    }

}

impl Guard<RwLockWriteGuard<'static, Store>> {

    fn dep(&self, key: &Key) -> Option<&Dep> {
        match key {
            $(Key::$upper(id) => match self.0.$lower.get(id) {
                Some(tree) => Some(&tree.dep),
                None => None,
            },)*
        }
    }

    fn set(&self, dep: Dep) -> Dep {
        let set = dep.set.into_iter().filter(|(key, res)| {
            self.dep(key).map_or(true, |dep| match res {
                Res::End => !dep.set.is_empty(),
                Res::Art => false,
            })
        }).collect();
        Dep{ set }
    }

    fn pop(&mut self, key: &Key, res: Res) -> Vec<Key> {
        let mut vec = Vec::new();
        $(vec.extend(self.0.$lower.iter_mut().filter_map(|(id, val)| {
            val.dep.pop(key, res).and_then(|pop| match pop {
                Res::End => Some(Key::$upper(id.clone())),
                Res::Art => None,
            })
        }));)*
        vec
    }

    fn put(&mut self, key: &Key, res: Res) -> Vec<Key> {
        let mut new = self.pop(key, res);
        let mut all = match res {
            Res::End => vec![key.clone()],
            Res::Art => vec![],
        };
        while !new.is_empty() {
            let add = new.iter().flat_map(|key| {
                self.pop(key, Res::End)
            }).collect();
            all.append(&mut new);
            new = add;
        }
        all.append(&mut new);
        all
    }

    $(pub fn $lower(mut self, val: $table) -> Result<TokenStream>
    where $table: Cache + Safe<Error = syn::Error>
    {
        let id = val.id()?;
        let dep = val.dep()?;
        let safe = val.send()?;

        let dep = self.set(dep);
        let key = Key::$upper(id.clone());
        let vec = self.put(&key, dep.res());

        match self.0.$lower.entry(id) {
            map::Entry::Occupied(entry) => {
                let key = Key::$upper(entry.key().clone());
                let msg = format!("duplicate definition: {key}\n\
                    note: cannot #[derive({})] on multiple structs with the same identifier",
                    stringify!($upper));
                return Err(syn::Error::new(Span::call_site(), msg));
            }
            map::Entry::Vacant(entry) => {
                entry.insert(Tree {
                    val: safe,
                    dep,
                });
            }
        }

        drop(self);
        let mut val = Some(val);
        let mut out = TokenStream::new();
        let read = cache::fetch();

        for put in vec {
            let put = match put == key {
                false => read.call(&put)?,
                true => match val.take() {
                    None => read.call(&put)?,
                    Some(val) => val.call()?,
                }
            };
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
