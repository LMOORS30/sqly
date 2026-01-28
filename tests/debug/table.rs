use sqly::derive::*;



#[derive(Table)]
#[sqly(table = "t", debug)]
struct T1;

#[derive(Table)]
#[sqly(table = "t", debug, flat)]
struct T2;

#[derive(Table)]
#[sqly(table = "t", debug, flat, flat_row)]
struct T3;

#[derive(Table)]
#[sqly(table = "t", debug, flat, flat_row, from_flat)]
struct T4;

#[derive(Table)]
#[sqly(table = "t", debug, flat, flat_row, from_flat, from_row)]
struct T5;

#[derive(Table)]
#[sqly(table = "t", debug, flat, flat_row, from_flat, from_row)]
struct T6 {
    #[sqly(key)]
    t1: (),
    t2: (),
}

#[derive(Table)]
#[sqly(table = "t", debug, delete, insert, select, update)]
struct T7 {
    #[sqly(key)]
    t1: i32,
    t2: i32,
}



fn main() {}
