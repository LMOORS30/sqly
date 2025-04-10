use super::*;



parse! {
    pub SelectTable {
        ((table)! (= safe::Paved)!),
        ((rename)? (= Rename)!),

        ((dynamic)?),
        ((optional)?),
        ((filter)* (= String)+),
        ((returning as "")? (= safe::Returning)?),

        ((krate as "crate")? (= syn::Path)!),
        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub SelectField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),
        ((optional)? (= bool)?),
        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl SelectTable {

    pub fn init(self) -> Result<Self> {
        for field in &self.fields {
            if let Some(skip) = &field.attr.skip {
                if !field.attr.filter.is_empty() {
                    let msg = "conflicting attributes: #[sqly(skip, filter)]";
                    return Err(syn::Error::new(skip.span, msg));
                }
            }
        }

        if let Some(span) = self.attr.returning.spany() {
            let msg = "impossible attribute: this should not exist";
            return Err(syn::Error::new(span, msg));
        }

        self.r#static()?;
        Ok(self)
    }

}
