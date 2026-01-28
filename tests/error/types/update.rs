use sqly::derive::*;



#[derive(Update)]
#[sqly(table = "")]
struct T1 {
    #[sqly(key)]
    t: (),
}

#[derive(Update)]
#[sqly(table = "")]
struct T2 {
    t: (),
}

#[derive(Update)]
#[sqly(table = "", keyless)]
struct T3 {
    #[sqly(key)]
    t1: (),
    t2: (),
}

#[derive(Update)]
#[sqly(table = "")]
struct T4 {
    #[sqly(key)]
    t1: (),
    t2: (),
    #[sqly(skip, key)]
    t3: (),
}

#[derive(Update)]
#[sqly(table = "")]
struct T5 {
    #[sqly(key)]
    t1: (),
    t2: (),
    #[sqly(skip, update = "")]
    t3: (),
}

#[derive(Update)]
#[sqly(table = "")]
struct T6 {
    #[sqly(key)]
    t1: (),
    t2: (),
    #[sqly(key, update = "")]
    t3: (),
}

#[derive(Update)]
#[sqly(table = "")]
struct T7 {
    #[sqly(key)]
    t1: (),
    t2: (),
    #[sqly(filter = "")]
    t3: (),
}



fn main() {}
