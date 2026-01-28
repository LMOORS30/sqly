use sqly::derive::*;



#[derive(Select)]
#[sqly(table = Option<()>)]
struct T1;

#[derive(Insert)]
#[sqly(table = Self)]
struct T2 {
    t: (),
}



mod a {
    use super::*;
    #[derive(Table)]
    #[sqly(table = "", from_row)]
    struct T1;
    #[derive(Select)]
    #[sqly(table = T1)]
    struct T2;
}

mod b {
    use super::*;
    #[derive(Table)]
    #[sqly(table = "", from_row)]
    struct T1;
    #[derive(Select)]
    #[sqly(table = T1)]
    struct T2;
}


#[derive(Select)]
#[sqly(table = Missing)]
struct T3;



#[derive(Delete)]
#[sqly(table = "", filter = "$t$")]
struct T4 {
    t: (),
}

#[derive(Delete)]
#[sqly(table = "", filter = "$$$_$")]
struct T5 {
    t: (),
}

#[derive(Insert)]
#[sqly(table = "")]
struct T6 {
    #[sqly(insert = "{${ it }}")] 
    t: (),
}



fn main() {}
