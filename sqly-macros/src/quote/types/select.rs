use super::*;



impl TryFrom<SelectTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: SelectTable) -> Result<TokenStream> {
        cache::store().select(table)
    }
}

impl Cache for SelectTable {

    fn id(&self) -> Result<Id> {
        Id::try_from(&self.ident)
    }

    fn dep(&self) -> Result<Dep> {
        let mut dep = Dep::new();
        let table = &self.attr.table.data.data;
        dep.end(Key::Table(table.try_into()?));
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}



impl SelectTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = &self.attr.table.data.data;
        let query = self.query()?;
        let ident = &self.ident;
        let res = result!['q];

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Select for #ident {
                type Table = #table;
                type Query<'q> = #res;
                fn select(&self) -> Self::Query<'_> {
                    #query
                }
            }
        })
    }

}



impl SelectTable {

    #[cfg(feature = "postgres")]
    pub fn query(&self) -> Result<TokenStream> {
        if self.attr.print.is_some() {
            println!("{}::query!(\n\tunimplemented!()\n)", self.ident);
        }

        let span = proc_macro2::Span::call_site();
        let msg = "select is not yet implemented";
        Err(syn::Error::new(span, msg))
    }

}
