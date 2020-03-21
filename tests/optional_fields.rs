use building::Builder;
use std::vec::Vec;

#[derive(Builder)]
pub struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
}

fn passing_optional_value() {
    let product = Product::builder()
        .id(12)
        .name("Foo".to_owned())
        .description("The product description".to_owned())
        .tags(vec!["foo".to_owned(), "bar".to_owned()])
        .build()
        .unwrap();

    assert_eq!(12, product.id);
    assert_eq!("Foo".to_owned(), product.name);
    assert_eq!(Some("The product description".to_owned()), product.description);
    assert_eq!(vec!["foo".to_owned(), "bar".to_owned()], product.tags);
}

fn omitting_optional_value() {
    let product = Product::builder()
        .id(12)
        .name("Foo".to_owned())
        .tags(vec!["foo".to_owned(), "bar".to_owned()])
        .build()
        .unwrap();

    assert_eq!(12, product.id);
    assert_eq!("Foo".to_owned(), product.name);
    assert_eq!(None, product.description);
    assert_eq!(vec!["foo".to_owned(), "bar".to_owned()], product.tags);
}

fn main() {
    passing_optional_value();
    omitting_optional_value();
}
