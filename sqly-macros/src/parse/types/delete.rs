use super::*;



parse! {
    pub DeleteTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub DeleteField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl DeleteTable {

    pub fn init(self) -> Result<Self> {
        if self.fields()?.next().is_none() {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing delete key";
            return Err(syn::Error::new(span, msg));
        }

        Ok(self)
    }

}
