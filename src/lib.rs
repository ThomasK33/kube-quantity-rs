#![forbid(unsafe_code)]

mod parser;

use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use parser::parse_quantity_string;

pub use parser::{ParseQuantityError, ParsedQuantity};

impl TryFrom<Quantity> for ParsedQuantity {
    type Error = ParseQuantityError;

    fn try_from(value: Quantity) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&Quantity> for ParsedQuantity {
    type Error = ParseQuantityError;

    fn try_from(value: &Quantity) -> Result<Self, Self::Error> {
        parse_quantity_string(&value.0).map(|(_, quantity)| quantity)
    }
}
