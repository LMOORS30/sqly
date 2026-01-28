use sqly::derive::*;



#[derive(Insert)]
#[sqly(table = "")]
struct T1;

#[derive(Insert)]
#[sqly(table = "")]
struct T2 {
    #[sqly(skip, insert = "")]
    t: (),
}



fn main() {}
