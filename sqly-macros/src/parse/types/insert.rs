use super::*;



parse! {
    pub InsertTable {
        ((table)! (= safe::Paved)!),
        ((rename)? (= Rename)!),

        ((dynamic)?),
        ((optional)?),
        ((returning)? (= safe::Returning)?),

        ((krate as "crate")? (= syn::Path)!),
        ((unchecked)?),
        ((print)?),
        ((debug)?),
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
            let span = Span::call_site();
            let msg = "incomplete query: missing insert value";
            return Err(syn::Error::new(span, msg));
        }

        self.r#static()?;
        Ok(self)
    }

}
