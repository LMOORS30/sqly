use super::*;



parse! {
    pub DeleteTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((print)?),
        ((debug)?),
    }
    pub DeleteField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

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
