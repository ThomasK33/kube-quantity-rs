#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

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

impl From<ParsedQuantity> for Quantity {
    fn from(value: ParsedQuantity) -> Self {
        Self(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;

    use crate::{ParseQuantityError, ParsedQuantity};

    #[test]
    fn test_quantity_addition_external() {
        let q1: Result<ParsedQuantity, _> = Quantity("1Ki".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("1Ki".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let q3: ParsedQuantity = q1.unwrap() + q2.unwrap();

        let q3: Quantity = q3.into();

        assert_eq!(q3.0, "2Ki");
    }

    #[test]
    fn test_quantity_addition_external_2() {
        let q1: Result<ParsedQuantity, _> = Quantity("1".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("500m".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let q3: ParsedQuantity = q1.unwrap() + q2.unwrap();

        let q3: Quantity = q3.into();

        assert_eq!(q3.0, "1500m");
    }

    #[test]
    fn test_quantity_subtraction_external_2() {
        let q1: Result<ParsedQuantity, _> = Quantity("1".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("500m".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let q3: ParsedQuantity = q1.unwrap() - q2.unwrap();

        let q3: Quantity = q3.into();

        assert_eq!(q3.0, "500m");
    }

    #[test]
    fn test_failing_parsing() {
        let q: Result<ParsedQuantity, ParseQuantityError> =
            Quantity("1.5.0".to_string()).try_into();

        assert!(q.is_err());
        assert_eq!(q.unwrap_err().to_string(), "quantity parsing failed");
    }
}
