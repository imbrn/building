use building::Builder;

#[derive(Builder)]
pub struct Product {
    id: i64,
    name: String,
    tags: Vec<String>,
}

fn main() {
    let mut builder = Product::builder();
    builder.id(12);
    builder.name("Foo".to_owned());
    builder.tags(vec!["foo".to_owned(), "bar".to_owned()]);

    let product = builder.build().unwrap();

    assert_eq!(12, product.id);
    assert_eq!("Foo", product.name);
    assert_eq!(vec!["foo".to_owned(), "bar".to_owned()], product.tags);
}
