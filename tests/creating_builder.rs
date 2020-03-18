use building::Builder;

#[derive(Builder)]
struct Product {
}

#[test]
fn should_create_builder_from_buildable() {
    let _builder = Product::builder();
}
