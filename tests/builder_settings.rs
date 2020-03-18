use building::Builder;

#[derive(Builder)]
struct BasicProduct {
    id: i64,
    name: String,
}

#[test]
fn basic_settings() {
    let mut builder = BasicProduct::builder();
    builder.id(12);
    builder.name("Foo".to_owned());
}
