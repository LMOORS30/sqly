use super::*;



parse! {
    pub SelectTable {
        ((table)! (= syn::Path)!),
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
        Ok(self)
    }

}
