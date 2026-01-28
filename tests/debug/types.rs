use sqly::derive::*;



#[derive(Delete)]
#[sqly(table = T3, debug)]
struct T1 {
    t1: i32,
}

#[derive(Insert)]
#[sqly(table = T3, debug)]
struct T2 {
    t2: i32,
}

#[derive(Table)]
#[sqly(table = "t", debug, from_row)]
struct T3 {
    t3: i32,
}

#[derive(Select)]
#[sqly(table = T3, debug)]
struct T4 {
    t4: i32,
}

#[derive(Update)]
#[sqly(table = T3, debug)]
struct T5 {
    #[sqly(key)]
    k5: i32,
    v5: i32,
}



fn main() {}
