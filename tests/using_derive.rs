use building::Builder;

#[derive(Builder)]
struct Product {
}

#[test]
fn should_work() {
    let _product = Product{};
}
