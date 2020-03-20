use building::Builder;

#[derive(Builder)]
pub struct Product {
}

fn main() {
    let _builder = Product::builder();
}
