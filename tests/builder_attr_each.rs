use building::Builder;
use std::vec::Vec;

#[derive(Builder)]
pub struct Product {
    id: i32,
    code: Option<u32>,
    name: String,
    description: Option<String>,
    #[builder(each = "tag")]
    tags: Vec<String>,
}

fn passing_vector_along_with_each() {
    let product = Product::builder()
        .id(12)
        .code(654321)
        .name("Foo".to_owned())
        .description("Bar".to_owned())
        .tags(vec!["foo".to_owned(), "bar".to_owned()])
        .tag("baz".to_owned())
        .build()
        .unwrap();

    assert_eq!(12, product.id);
    assert_eq!(Some(654321), product.code);
    assert_eq!("Foo".to_owned(), product.name);
    assert_eq!(Some("Bar".to_owned()), product.description);
    assert_eq!(
        vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
        product.tags
    );
}

fn passing_only_each() {
    let product = Product::builder()
        .id(12)
        .code(654321)
        .name("Foo".to_owned())
        .description("Bar".to_owned())
        .tag("foo".to_owned())
        .tag("bar".to_owned())
        .build()
        .unwrap();

    assert_eq!(12, product.id);
    assert_eq!(Some(654321), product.code);
    assert_eq!("Foo".to_owned(), product.name);
    assert_eq!(Some("Bar".to_owned()), product.description);
    assert_eq!(vec!["foo".to_owned(), "bar".to_owned()], product.tags);
}

fn main() {
    passing_vector_along_with_each();
    passing_only_each();
}
