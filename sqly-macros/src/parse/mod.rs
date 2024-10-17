#[macro_use]
mod rules;
mod types;
mod base;

pub use rules::*;
pub use types::*;



pub fn typath(mut ty: &syn::Type) -> Option<&syn::Path> {
    loop {
        match ty {
            syn::Type::Path(path) => break Some(&path.path),
            syn::Type::Group(group) => ty = &group.elem,
            syn::Type::Paren(paren) => ty = &paren.elem,
            _ => break None,
        }
    }
}

pub fn optype(ty: &syn::Type) -> Option<(&syn::Path, &syn::Type)> {
    let path = match typath(ty) {
        Some(path) => path,
        None => return None,
    };
    let segment = match path.segments.last() {
        Some(segment) => segment,
        None => return None,
    };
    if segment.ident != "Option" {
        return None;
    }
    let arguments = match &segment.arguments {
        syn::PathArguments::AngleBracketed(gen) => &gen.args,
        _ => return None,
    };
    let argument = match arguments.len() {
        1 => &arguments[0],
        _ => return None,
    };
    let ty = match argument {
        syn::GenericArgument::Type(ty) => ty,
        _ => return None,
    };
    Some((path, ty))
}

pub fn argone(path: &syn::Path) -> syn::Path {
    let mut path = path.clone();
    if let Some(segment) = path.segments.last_mut() {
        segment.arguments = syn::PathArguments::None;
    }
    path
}



pub fn unfer(expr: &syn::Expr) -> Option<syn::Expr> {
    match expr {
        syn::Expr::Group(group) => match unfer(&group.expr) {
            Some(expr) => Some(syn::Expr::Group(syn::ExprGroup {
                expr: Box::new(expr),
                attrs: group.attrs.clone(),
                group_token: group.group_token.clone(),
            })),
            None => None,
        },
        syn::Expr::Paren(paren) => match unfer(&paren.expr) {
            Some(expr) => Some(syn::Expr::Paren(syn::ExprParen {
                expr: Box::new(expr),
                attrs: paren.attrs.clone(),
                paren_token: paren.paren_token.clone(),
            })),
            None => None,
        },
        syn::Expr::Cast(cast) => match &*cast.ty {
            syn::Type::Infer(_) => Some((*cast.expr).clone()),
            _ => None,
        },
        _ => None,
    }
}
