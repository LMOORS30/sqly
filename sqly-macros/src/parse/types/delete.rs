use super::*;



parse! {
    pub DeleteTable {
        ((table)! (= safe::Paved)!),
        ((rename_all)? (= Rename)!),

        ((dynamic)?),
        ((optional)?),
        ((keyless)?),
        ((filter)* (= String)+),
        ((returning)? (= safe::Returning)?),

        ((unchecked)? (= Checks)?),
        ((krate as "crate")? (= syn::Path)!),
        ((print)? (= Print)?),
        ((debug)? (= Print)?),
    }
    pub DeleteField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),
        ((optional)? (= bool)?),
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

        if let Some(keyless) = &self.attr.keyless {
            if self.fields().next().is_some() {
                let msg = "conflicting attribute: #[sqly(keyless)] with keys\n\
                    help: remove #[sqly(keyless)]";
                return Err(syn::Error::new(keyless.span, msg));
            }
        } else {
            if self.fields().next().is_none() {
                let msg = "incomplete query: missing delete key\n\
                    help: use #[sqly(keyless)] if intended";
                return Err(syn::Error::new(Span::call_site(), msg));
            }
        }

        self.r#static()?;
        Ok(self)
    }

}
