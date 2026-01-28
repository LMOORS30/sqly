use sqly::derive::*;



#[derive(Table, Delete, Insert, Select, Update)]
#[sqly(table = "")]
enum T1 {}

#[derive(Table, Delete, Insert, Select, Update)]
#[sqly(table = "")]
struct T2(());

#[derive(Table, Delete, Insert, Select, Update)]
struct T3;

#[derive(Delete)]
#[sqly(table = "", table = "")]
struct T4;

#[derive(Update)]
#[sqly(table = "")]
struct T5 {
    #[sqly(key)]
    #[sqly(key)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T6 {
    #[sqly(foreign = "")]
    #[sqly(foreign)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T7 {
    #[sqly(foreign)]
    #[sqly(foreign = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T8 {
    #[sqly(foreign, foreign)]
    t: (),
}

#[derive(Table)]
#[sqly(unknown)]
struct T9;

#[derive(Select)]
#[sqly(table = T)]
struct T10 {
    #[sqly(unknown = ())]
    t: (),
}

#[derive(Insert)]
#[sqly(rename_all = "unknown")]
struct T11;

#[derive(Table)]
#[sqly(unchecked = unknown)]
struct T12;

#[derive(Table)]
#[sqly(print = unknown)]
struct T13;

#[derive(Table)]
#[sqly(table = "")]
struct T14 {
    #[sqly(key = unknown)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T15 {
    #[sqly(skip = unknown)]
    t: (),
}



fn main() {}
