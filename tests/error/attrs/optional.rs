use sqly::derive::*;



#[derive(Table)]
#[sqly(table = "", update)]
#[sqly(update_dynamic)]
struct T1 {
    #[sqly(key)]
    t1: (),
    t2: (),
}

#[derive(Table)]
#[sqly(table = "", insert)]
#[sqly(insert_optional)]
struct T2 {
    t: (),
}

#[derive(Select)]
#[sqly(table = T)]
#[sqly(optional, dynamic)]
struct T3 {
    t1: (),
}

#[derive(Delete)]
#[sqly(table = "")]
#[sqly(optional)]
struct T4 {
    t: Option<()>,
}



fn main() {}
