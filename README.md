# building
Rust procedural macro for deriving Builder in structs.

> This project was developed while I was playing with the Builder section of [this workshop](https://github.com/dtolnay/proc-macro-workshop) teaching about Rust procedural macros.


## Simple usage example

```rust
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

fn main() {
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
```

## License

[MIT License](https://opensource.org/licenses/MIT)
