use building::Builder;

#[derive(Debug, PartialEq, Builder)]
pub struct Product {
    id: i64,
    name: String,
    tags: Vec<String>,
}

fn working() {
    let product = Product::builder()
        .id(12)
        .name("Foo".to_owned())
        .tags(vec!["foo".to_owned(), "bar".to_owned()])
        .build()
        .unwrap();

    assert_eq!(12, product.id);
    assert_eq!("Foo", product.name);
    assert_eq!(vec!["foo".to_owned(), "bar".to_owned()], product.tags);
}

fn failing() {
    let product = Product::builder()
        .id(12)
        .tags(vec!["foo".to_owned(), "bar".to_owned()])
        .build();

    assert_eq!(product, Err("Failed to build field".to_owned()));
}

fn main() {
    working();
    failing();
}
