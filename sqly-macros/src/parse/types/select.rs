use super::*;



parse! {
    pub SelectTable {
        ((table)! (= safe::Paved)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub SelectField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),
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

        Ok(self)
    }

}
