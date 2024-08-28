use super::*;



parse! {
    pub SelectTable {
        ((table)! (= Path)!),
        ((table_name)! (= String)!),
        ((rename)? (= Rename)!),

        ((print)?),
        ((debug)?),
    }
    pub SelectField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((skip)?),
    }
}



impl SelectTable {

    pub fn init(self) -> Result<Self> {
        if self.fields()?.next().is_none() {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing select key";
            return Err(syn::Error::new(span, msg));
        }

        Ok(self)
    }

}
