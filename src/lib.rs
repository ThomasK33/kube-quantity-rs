#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod format;
mod parser;
mod quantity;
mod scale;
mod utils;

use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use parser::parse_quantity_string;

pub use parser::ParseQuantityError;
pub use quantity::ParsedQuantity;

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

impl TryFrom<&str> for ParsedQuantity {
    type Error = ParseQuantityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_quantity_string(value).map(|(_, quantity)| quantity)
    }
}

impl TryFrom<String> for ParsedQuantity {
    type Error = ParseQuantityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_quantity_string(&value).map(|(_, quantity)| quantity)
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
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;

    use crate::format::Format;
    use crate::scale::Scale;
    use crate::{ParseQuantityError, ParsedQuantity};

    #[test]
    fn test_quantity_addition() {
        let q1: Result<ParsedQuantity, _> = Quantity("1Ki".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("1Ki".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let q3: ParsedQuantity = q1.unwrap() + q2.unwrap();

        let q3: Quantity = q3.into();

        assert_eq!(q3.0, "2Ki");
    }

    #[test]
    fn test_quantity_addition_2() {
        let q1: Result<ParsedQuantity, _> = Quantity("1".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("500m".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let q3: ParsedQuantity = q1.unwrap() + q2.unwrap();

        let q3: Quantity = q3.into();

        assert_eq!(q3.0, "1500m");
    }

    #[test]
    fn test_quantity_addition_assign() {
        let q1: Result<ParsedQuantity, _> = Quantity("5M".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("7M".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let mut q1 = q1.unwrap();
        q1 += q2.unwrap();

        let q1: Quantity = q1.into();

        assert_eq!(q1.0, "12M");
    }

    #[test]
    fn test_quantity_subtraction() {
        let q1: Result<ParsedQuantity, _> = Quantity("1".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("500m".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let q3: ParsedQuantity = q1.unwrap() - q2.unwrap();

        let q3: Quantity = q3.into();

        assert_eq!(q3.0, "500m");
    }

    #[test]
    fn test_quantity_subtraction_assign() {
        let q1: Result<ParsedQuantity, _> = Quantity("10Gi".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("500Mi".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let mut q1 = q1.unwrap();
        q1 -= q2.unwrap();

        let q1: Quantity = q1.into();

        assert_eq!(q1.0, "9740Mi");
    }

    #[test]
    fn test_quantity_subtraction_assign_2() {
        let q1: Result<ParsedQuantity, _> = Quantity("10G".to_string()).try_into();
        let q2: Result<ParsedQuantity, _> = Quantity("500M".to_string()).try_into();

        assert!(q1.is_ok());
        assert!(q2.is_ok());

        let mut q1 = q1.unwrap();
        q1 -= q2.unwrap();

        let q1: Quantity = q1.into();

        assert_eq!(q1.0, "9500M");
    }

    #[test]
    fn test_failing_parsing() {
        let q: Result<ParsedQuantity, ParseQuantityError> =
            Quantity("1.5.0".to_string()).try_into();

        assert!(q.is_err());
        assert_eq!(q.unwrap_err().to_string(), "quantity parsing failed");
    }

    #[test]
    fn test_div() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(2.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q1 = ParsedQuantity {
            value: Decimal::from_f32(4.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = Decimal::from_f32(2.0).unwrap();

        let result = q1 / q2;

        assert_eq!(result, exp_result);
    }

    #[test]
    fn test_div_decimal_f32() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(2.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q1 = ParsedQuantity {
            value: Decimal::from_f32(5.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = Decimal::from_f32(2.5).unwrap();

        let result = q1 / q2;

        assert_eq!(result, exp_result);
    }

    #[test]
    fn test_div_decimal_u8() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(2.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q1 = ParsedQuantity {
            value: Decimal::from_f32(6.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = 3;

        let result = q1 / q2;

        assert_eq!(result, exp_result);
    }

    #[test]
    fn test_div_assign() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(2.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let mut q1 = ParsedQuantity {
            value: Decimal::from_f32(4.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = Decimal::from_f32(2.0).unwrap();

        q1 /= q2;

        assert_eq!(q1, exp_result);
    }

    #[test]
    fn test_mul() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(8.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q1 = ParsedQuantity {
            value: Decimal::from_f32(4.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = Decimal::from_f32(2.0).unwrap();

        let result = q1 * q2;

        assert_eq!(result, exp_result);
    }

    #[test]
    fn test_mul_f32() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(10.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q1 = ParsedQuantity {
            value: Decimal::from_f32(5.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = Decimal::from_f32(2.0).unwrap();

        let result = q1 * q2;

        assert_eq!(result, exp_result);
    }

    #[test]
    fn test_mul_u8() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(9.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q1 = ParsedQuantity {
            value: Decimal::from_f32(4.5).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = 2;

        let result = q1 * q2;

        assert_eq!(result, exp_result);
    }

    #[test]
    fn test_mul_assign() {
        let exp_result = ParsedQuantity {
            value: Decimal::from_f32(8.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let mut q1 = ParsedQuantity {
            value: Decimal::from_f32(4.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = Decimal::from_f32(2.0).unwrap();

        q1 *= q2;

        assert_eq!(q1, exp_result);
    }
}
