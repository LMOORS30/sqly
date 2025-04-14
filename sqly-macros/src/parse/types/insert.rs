use super::*;



parse! {
    pub InsertTable {
        ((table)! (= safe::Paved)!),
        ((rename)? (= Rename)!),

        ((dynamic)?),
        ((optional)?),
        ((returning)? (= safe::Returning)?),

        ((unchecked)?),
        ((krate as "crate")? (= syn::Path)!),
        ((print)? (= Print)?),
        ((debug)? (= Print)?),
    }
    pub InsertField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((insert)* (= String)+),
        ((optional)? (= bool)?),
        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl InsertTable {

    pub fn init(self) -> Result<Self> {
        for field in &self.fields {
            if let Some(skip) = &field.attr.skip {
                if !field.attr.insert.is_empty() {
                    let msg = "conflicting attributes: #[sqly(skip, insert)]";
                    return Err(syn::Error::new(skip.span, msg));
                }
            }
        }

        if self.fields().next().is_none() {
            let msg = "incomplete query: missing insert value";
            return Err(syn::Error::new(Span::call_site(), msg));
        }

        self.r#static()?;
        Ok(self)
    }

}
