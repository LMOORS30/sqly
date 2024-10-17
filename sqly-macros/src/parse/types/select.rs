use super::*;



parse! {
    pub SelectTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub SelectField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl SelectTable {

    pub fn init(self) -> Result<Self> {
        Ok(self)
    }

}
