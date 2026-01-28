use sqly::derive::*;



#[derive(Table)]
#[sqly(table = "")]
struct T1;

#[derive(Select)]
#[sqly(table = "")]
struct T2;

#[derive(Select)]
#[sqly(table = T1)]
struct T3;

#[derive(Select)]
#[sqly(table = T)]
struct T4 {
    #[sqly(skip, filter = "")]
    t: (),
}



fn main() {}
