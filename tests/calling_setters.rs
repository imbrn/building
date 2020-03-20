#![allow(unused)]

use building::Builder;

#[derive(Builder)]
pub struct Product {
    id: i32,
    name: String,
    tags: Vec<String>,
}

fn mutable_builder() {
    let mut builder = Product::builder();
    builder.id(12);
    builder.name("Foo".to_owned());
    builder.tags(vec!["foo".to_owned(), "bar".to_owned()]);
}

fn chaining_setters() {
    let _builder = Product::builder()
        .id(12)
        .name("Foo".to_owned())
        .tags(vec!["foo".to_owned(), "bar".to_owned()]);
}

fn main() {
    mutable_builder();
    chaining_setters();
}
