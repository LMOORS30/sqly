use sqly::derive::*;

/*

TODO

*/

#[derive(Table)]
#[sqly(table = "books")]
struct Book {
    #[sqly(key)]
    id: i32,
    title: String,
}

#[derive(Table)]
#[sqly(table = "pages")]
#[sqly(insert, update, select)]
#[sqly(delete = DeleteAllPages)]
struct Page {
    #[sqly(key, foreign)]
    book: Book,
    #[sqly(key, skip = delete)]
    page_number: i32,
    content: String,
    #[sqly(skip = update)]
    read: bool,
}

#[derive(Select)]
#[sqly(table = Page)]
struct GetBookPages {
    book_id: i32,
}

#[derive(Update)]
#[sqly(table = Page)]
struct MarkAsRead {
    #[sqly(key)]
    book_id: i32,
    #[sqly(key)]
    page_number: i32,
    read: bool,
}

#[test]
fn c36_a_lot_at_once() {}
