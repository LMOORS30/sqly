use super::*;



parse! {
    pub InsertTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub InsertField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl InsertTable {

    pub fn init(self) -> Result<Self> {
        if self.fields()?.next().is_none() {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing insert value";
            return Err(syn::Error::new(span, msg));
        }

        Ok(self)
    }

}
