use sqly::derive::*;



#[derive(Table)]
#[sqly(table = "")]
#[sqly(delete_derive = Clone)]
struct T1;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(insert_visibility = ,)]
struct T2;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(select_filter = "")]
struct T3;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(delete_keyless)]
struct T4;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(insert_dynamic)]
struct T5;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(update_optional)]
struct T6;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(delete_returning)]
struct T7;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(from_flat)]
struct T8;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(try_from_flat)]
struct T9;

#[derive(Table)]
#[sqly(table = "")]
#[sqly(flat_row)]
struct T10;

#[derive(Table)]
#[sqly(table = "")]
struct T11 {
    #[sqly(key = update)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T12 {
    #[sqly(skip = from_row, from_row)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T13 {
    #[sqly(skip, key)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", delete)]
struct T14 {
    #[sqly(skip, key = delete)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", select)]
struct T15 {
    #[sqly(skip = select, key = select)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T16 {
    #[sqly(insert = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T17 {
    #[sqly(select_filter = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T18 {
    #[sqly(update_optional = false)]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", update)]
struct T19 {
    #[sqly(skip, update = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", delete)]
struct T20 {
    #[sqly(delete_filter = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", insert)]
struct T21 {
    #[sqly(insert_optional, skip = insert)]
    t1: (),
    t2: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T22 {
    #[sqly(filter = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", update)]
struct T23 {
    #[sqly(update_filter = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T24 {
    #[sqly(skip, foreign)]
    t: i32,
}

#[derive(Table)]
#[sqly(table = "")]
struct T25 {
    #[sqly(foreign, skip = from_row)]
    t: i32,
}

#[derive(Table)]
#[sqly(table = "")]
struct T26 {
    #[sqly(foreign, select = "")]
    t: i32,
}

#[derive(Table)]
#[sqly(table = "")]
struct T27 {
    #[sqly(target = "")]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", flat, from_flat, try_from_flat)]
struct T28 {
    #[sqly(from = (), try_from = ())]
    t: (),
}

#[derive(Table)]
#[sqly(table = "")]
struct T29 {
    #[sqly(from = (), try_from = ())]
    t: (),
}

#[derive(Table)]
#[sqly(table = "", flat, from_flat)]
struct T30 {
    #[sqly(try_from = ())]
    t: (),
}



fn main() {}
