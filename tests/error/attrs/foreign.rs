use sqly::derive::*;



#[derive(Table)]
#[sqly(table = "")]
struct T1 {
    t: Option<()>,
}

#[derive(Table)]
#[sqly(table = "")]
struct T2 {
    #[sqly(foreign)]
    t: Option<&'static str>,
}

#[derive(Table)]
#[sqly(table = "")]
struct T3 {
    #[sqly(foreign, key, column = "t")]
    t1: T1,
    #[sqly(key, column = "t")]
    t2: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T4 {
    #[sqly(foreign)]
    t: T3,
}

#[derive(Table)]
#[sqly(table = "")]
struct T5 {
    #[sqly(foreign, target = t)]
    t: T3,
}

#[derive(Table)]
#[sqly(table = "", insert)]
struct T6 {
    #[sqly(foreign, target = "t")]
    t: T3,
}

#[derive(Table)]
#[sqly(table = "")]
struct T7 {
    #[sqly(foreign, target = "t")]
    t1: T1,
    t2: Option<()>,
}

#[derive(Table)]
#[sqly(table = "", from_row)]
struct T8 {
    #[sqly(foreign, target = t1)]
    t: Option<T7>,
}



fn main() {}
