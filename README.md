# kube_quantity - Kubernetes Quantity Parser

[![Crates.io](https://img.shields.io/crates/v/kube_quantity)](https://crates.io/crates/kube_quantity)

`kube_quantity` is a library adding arithmetic operations to the [`Quantity`](https://arnavion.github.io/k8s-openapi/v0.17.x/k8s_openapi/apimachinery/pkg/api/resource/struct.Quantity.html#) type from the [`k8s-openapi`](https://crates.io/crates/k8s-openapi) crate.

## Installation

Run the following Cargo command in your project directory to add the latest stable version:

```bash
cargo add kube_quantity
```

Or add the following line to your Cargo.toml:

```toml
[dependencies]
kube_quantity = "0.2.0"
```

## Upgrading

Please check the [CHANGELOG](https://github.com/ThomasK33/kube-quantity-rs/blob/main/CHANGELOG.md) when upgrading.

## Usage

## Parsing of quantities

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube_quantity::{ParseQuantityError, ParsedQuantity};

// Parse directly from &str
let quantity = "1Ki";
let quantity: Result<ParsedQuantity, ParseQuantityError> = quantity.try_into();
assert_eq!(quantity.unwrap().to_string(), "1Ki");

// Parse from a String
let quantity = String::from("2Mi");
let quantity: Result<ParsedQuantity, ParseQuantityError> = quantity.try_into();
assert_eq!(quantity.unwrap().to_string(), "2Mi");

// Parse from a `k8s_openapi` Quantity
let quantity = Quantity("2.5Gi".to_string());
let quantity: Result<ParsedQuantity, ParseQuantityError> = quantity.try_into();
assert_eq!(quantity.unwrap().to_string(), "2.5Gi");
```

### Addition of quantities

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube_quantity::{ParseQuantityError, ParsedQuantity};

// Try parsing k8s quantities
let q1: Result<ParsedQuantity, ParseQuantityError> = Quantity("1Ki".to_string()).try_into();
let q2: Result<ParsedQuantity, ParseQuantityError> = Quantity("2Ki".to_string()).try_into();

// Add parsed quantities
let q3: ParsedQuantity = q1.unwrap() + q2.unwrap();
// Convert parsed quantity back into a k8s quantity
let q3: Quantity = q3.into();

assert_eq!(q3.0, "3Ki");
```

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube_quantity::{ParseQuantityError, ParsedQuantity};

let q1: Result<ParsedQuantity, ParseQuantityError> = Quantity("5M".to_string()).try_into();
let q2: Result<ParsedQuantity, ParseQuantityError> = Quantity("7M".to_string()).try_into();

let mut q1 = q1.unwrap();
q1 += q2.unwrap();

let q1: Quantity = q1.into();

assert_eq!(q1.0, "12M");

```

### Subtraction of quantities

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube_quantity::{ParseQuantityError, ParsedQuantity};

// Try parsing k8s quantities
let q1: Result<ParsedQuantity, ParseQuantityError> = Quantity("1M".to_string()).try_into();
let q2: Result<ParsedQuantity, ParseQuantityError> = Quantity("500k".to_string()).try_into();

// Subtract parsed quantities
let q3: ParsedQuantity = q1.unwrap() - q2.unwrap();
// Convert parsed quantity back into a k8s quantity
let q3: Quantity = q3.into();

assert_eq!(q3.0, "500k");
```

```rust
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube_quantity::{ParseQuantityError, ParsedQuantity};

// Try parsing k8s quantities
let q1: Result<ParsedQuantity, ParseQuantityError> = Quantity("10G".to_string()).try_into();
let q2: Result<ParsedQuantity, ParseQuantityError> = Quantity("500M".to_string()).try_into();

let mut q1 = q1.unwrap();
q1 -= q2.unwrap();

let q1: Quantity = q1.into();

assert_eq!(q1.0, "9500M");
```

## License

Apache 2.0 licensed. See [LICENSE](https://github.com/ThomasK33/kube-quantity-rs/blob/main/LICENSE) for details.
