# k8s_quantity - Kubernetes Quantity Parser

`k8s_quantity` is a library adding arithmetic operations to the [`Quantity`](https://arnavion.github.io/k8s-openapi/v0.17.x/k8s_openapi/apimachinery/pkg/api/resource/struct.Quantity.html#) type from the [`k8s-openapi`](https://crates.io/crates/k8s-openapi) crate.

## Usage

### Addition of quantities

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_quantity::{ParseQuantityError, ParsedQuantity};

// Try parsing k8s quantities
let q1: Result<ParsedQuantity, ParseQuantityError> = Quantity("1Ki".to_string()).try_into();
let q2: Result<ParsedQuantity, ParseQuantityError> = Quantity("2Ki".to_string()).try_into();

// Add parsed quantities
let q3: ParsedQuantity = q1.unwrap() + q2.unwrap();
// Convert parsed quantity back into a k8s quantity
let q3: Quantity = q3.into();

assert_eq!(q3.0, "3Ki");
```

### Subtraction of quantities

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_quantity::{ParseQuantityError, ParsedQuantity};

// Try parsing k8s quantities
let q1: Result<ParsedQuantity, ParseQuantityError> = Quantity("1M".to_string()).try_into();
let q2: Result<ParsedQuantity, ParseQuantityError> = Quantity("500k".to_string()).try_into();

// Subtract parsed quantities
let q3: ParsedQuantity = q1.unwrap() - q2.unwrap();
// Convert parsed quantity back into a k8s quantity
let q3: Quantity = q3.into();

assert_eq!(q3.0, "500k");
```
