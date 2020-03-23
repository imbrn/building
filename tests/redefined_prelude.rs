use building::Builder;
use std::vec::Vec;

type Option = ();
type Some = ();
type None = ();
type Result = ();
type Box = ();

#[derive(Builder)]
pub struct Product {
    id: i32,
    name: std::option::Option<String>,
    #[builder(each = "tag")]
    tags: Vec<String>,
}

fn main() {
    let product = Product::builder()
        .id(12)
        .name("Foo".to_owned())
        .tag("foo".to_owned())
        .build()
        .unwrap();

    assert_eq!(12, product.id);
    assert_eq!(std::option::Option::Some("Foo".to_owned()), product.name);
    assert_eq!(vec!["foo".to_owned()], product.tags);
}
