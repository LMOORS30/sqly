use sqly::derive::*;



#[derive(Delete)]
#[sqly(table = "")]
struct T1;

#[derive(Delete)]
#[sqly(table = "", keyless)]
struct T2 {
    t: (),
}

#[derive(Delete)]
#[sqly(table = "")]
struct T3 {
    #[sqly(skip, filter = "")]
    t: (),
}



fn main() {}
