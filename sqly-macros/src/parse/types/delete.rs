use super::*;



parse! {
    pub DeleteTable {
        ((table)! (= safe::Paved)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),

        ((krate as "crate")? (= syn::Path)!),
        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub DeleteField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),
        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl DeleteTable {

    pub fn init(self) -> Result<Self> {
        for field in &self.fields {
            if let Some(skip) = &field.attr.skip {
                if !field.attr.filter.is_empty() {
                    let msg = "conflicting attributes: #[sqly(skip, filter)]";
                    return Err(syn::Error::new(skip.span, msg));
                }
            }
        }

        if self.attr.filter.is_empty() {
            if self.fields()?.next().is_none() {
                let span = Span::call_site();
                let msg = "incomplete query: missing delete key";
                return Err(syn::Error::new(span, msg));
            }
        }

        Ok(self)
    }

}
