use sqly::derive::*;



#[derive(Table)]
#[sqly(table = "")]
struct T1 {
    t: (),
}

#[derive(Table)]
#[sqly(table = "", from_row)]
struct T2 {
    #[sqly(foreign, target = t)]
    t: T1,
}

#[derive(Delete)]
#[sqly(table = "")]
#[sqly(returning)]
struct T3 {
    t: (),
}

#[derive(Insert)]
#[sqly(table = T1)]
#[sqly(returning)]
struct T4 {
    t: (),
}

#[derive(Update)]
#[sqly(table = "")]
#[sqly(returning = T2)]
struct T5 {
    #[sqly(key)]
    t1: (),
    t2: (),
}

#[derive(Table)]
#[sqly(table = "", insert)]
#[sqly(insert_returning = T2 { t })]
struct T6 {
    t: (),
}

#[derive(Table)]
#[sqly(table = "", delete)]
#[sqly(delete_returning = T1 { t1 })]
struct T7 {
    #[sqly(key)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", update)]
#[sqly(update_returning = { t })]
struct T8 {
    #[sqly(key)]
    t1: (),
    t2: (),
}



fn main() {}
