use super::*;



parse! {
    pub InsertTable {
        ((table)! (= Path)!),
        ((table_name)! (= String)!),
        ((rename)? (= Rename)!),

        ((print)?),
        ((debug)?),
    }
    pub InsertField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

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
