#![allow(unused)]

use building::Builder;

#[derive(Builder)]
pub struct Product {
    id: i32,
    name: String,
    tags: Vec<String>,
}

fn main() {
    let mut builder = Product::builder();
    builder.id(12);
    builder.name("Foo".to_owned());
    builder.tags(vec!["foo".to_owned(), "bar".to_owned()]);
}
