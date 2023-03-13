use std::{fmt::Display, ops::Add};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{eof, opt},
    number::complete::double,
    IResult,
};
use rust_decimal::prelude::*;
use thiserror::Error;

// --- Errors ---

#[derive(Debug, Error)]
pub enum ParseQuantityError {
    #[error("empty string")]
    EmptyString,

    #[error("parsing failed")]
    ParsingFailed(#[from] nom::Err<nom::error::Error<String>>),
}

// --- Types ---

// - Parser Quantity -

#[derive(Debug, Clone)]
pub struct ParsedQuantity {
    // The actual value of the quantity
    value: Decimal,
    // Scale used to indicate the base-10 exponent of the value
    scale: Scale,
    // Used to indicate the format of the suffix used
    format: Format,

    // The string representation of this quantity to avoid recalculation
    string_representation: String,
}

impl Display for ParsedQuantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string_representation,)
    }
}

impl Add for ParsedQuantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

// - Format -

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    BinarySI,        // e.g., 12Mi (12 * 2^20)
    DecimalExponent, // e.g., 12e6
    DecimalSI,       // e.g., 12M  (12 * 10^6)
}

// - Scale

/// Scale is used for getting and setting the base-10 scaled value. Base-2
/// scales are omitted for mathematical simplicity.
#[derive(PartialEq, Eq, Debug, Clone)]
enum Scale {
    Milli,
    One,
    Kilo,
    Mega,
    Giga,
    Tera,
    Peta,
    Exa,
}

// Returns a tuple indicating wether the exponent is positive and the exponent
// itself
impl From<Scale> for (bool, u32) {
    fn from(value: Scale) -> Self {
        // TODO: https://en.wikipedia.org/wiki/Kilobyte
        match value {
            Scale::Milli => (false, 1),
            Scale::One => (true, 0),
            Scale::Kilo => (true, 1),
            Scale::Mega => (true, 2),
            Scale::Giga => (true, 3),
            Scale::Tera => (true, 4),
            Scale::Peta => (true, 5),
            Scale::Exa => (true, 6),
        }
    }
}

// --- Functions ---

fn scale_format_to_string(scale: &Scale, format: &Format) -> String {
    match format {
        Format::BinarySI => match scale {
            Scale::Milli => "".to_owned(),
            Scale::One => "".to_owned(),
            Scale::Kilo => "Ki".to_owned(),
            Scale::Mega => "MI".to_owned(),
            Scale::Giga => "Gi".to_owned(),
            Scale::Tera => "Ti".to_owned(),
            Scale::Peta => "Pi".to_owned(),
            Scale::Exa => "Ei".to_owned(),
        },
        Format::DecimalSI => match scale {
            Scale::Milli => "m".to_owned(),
            Scale::One => "".to_owned(),
            Scale::Kilo => "k".to_owned(),
            Scale::Mega => "M".to_owned(),
            Scale::Giga => "G".to_owned(),
            Scale::Tera => "T".to_owned(),
            Scale::Peta => "P".to_owned(),
            Scale::Exa => "E".to_owned(),
        },
        Format::DecimalExponent => "e".to_owned(),
    }
}

// --- Parsers ---

pub(crate) fn parse_quantity_string(
    input: &str,
) -> Result<(&str, ParsedQuantity), ParseQuantityError> {
    if input.is_empty() {
        return Err(ParseQuantityError::EmptyString);
    }

    let original_input = input.to_owned();

    let error_mapper = |err: nom::Err<nom::error::Error<&str>>| match err {
        nom::Err::Incomplete(err) => nom::Err::Incomplete(err),
        nom::Err::Error(err) => nom::Err::Error(nom::error::Error {
            input: err.input.to_owned(),
            code: err.code,
        }),
        nom::Err::Failure(err) => nom::Err::Failure(nom::error::Error {
            input: err.input.to_owned(),
            code: err.code,
        }),
    };

    let (input, signed_number) = parse_signed_number(input).map_err(error_mapper)?;
    let (input, (format, scale)) = parse_suffix(input).map_err(error_mapper)?;
    let (input, _) = eof(input).map_err(error_mapper)?;

    Ok((
        input,
        ParsedQuantity {
            format,
            scale,
            string_representation: original_input,
            value: Decimal::from_f64(signed_number).unwrap_or_default(),
        },
    ))
}

fn parse_signed_number(input: &str) -> IResult<&str, f64> {
    // Default to true
    let (input, positive) =
        opt(parse_sign)(input).map(|(input, positive)| (input, positive.unwrap_or(true)))?;
    // Default num to 0.0
    let (input, num) = opt(double)(input).map(|(input, num)| (input, num.unwrap_or(0.0)))?;

    Ok((input, if positive { num } else { -num }))
}

fn parse_suffix(input: &str) -> IResult<&str, (Format, Scale)> {
    // If the input is empty, then in a previous step we have already parsed the number
    // and we can classify this as a decimal exponent, yet one is going to
    // set this to a decimal si for compatibility reasons
    if input.is_empty() {
        return Ok((input, (Format::DecimalSI, Scale::One)));
    }

    // In the case that the string is not empty, we need to parse the suffix
    let (input, si) = alt((
        tag("Ki"),
        tag("Mi"),
        tag("Gi"),
        tag("Ti"),
        tag("Pi"),
        tag("Ei"),
        tag("m"),
        tag("k"),
        tag("M"),
        tag("G"),
        tag("T"),
        tag("P"),
        tag("E"),
    ))(input)?;

    Ok((
        input,
        match si {
            "Ki" => (Format::BinarySI, Scale::Kilo),
            "Mi" => (Format::BinarySI, Scale::Mega),
            "Gi" => (Format::BinarySI, Scale::Giga),
            "Ti" => (Format::BinarySI, Scale::Tera),
            "Pi" => (Format::BinarySI, Scale::Peta),
            "Ei" => (Format::BinarySI, Scale::Exa),
            //
            "m" => (Format::DecimalSI, Scale::Milli),
            "" => (Format::DecimalSI, Scale::One),
            "k" => (Format::DecimalSI, Scale::Kilo),
            "M" => (Format::DecimalSI, Scale::Mega),
            "G" => (Format::DecimalSI, Scale::Giga),
            "T" => (Format::DecimalSI, Scale::Tera),
            "P" => (Format::DecimalSI, Scale::Peta),
            "E" => (Format::DecimalSI, Scale::Exa),
            //
            _ => (Format::DecimalSI, Scale::One),
        },
    ))
}

fn parse_sign(input: &str) -> IResult<&str, bool> {
    let (input, sign) = one_of("+-")(input)?;
    Ok((input, sign == '+'))
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_string_parsing() {
        let quantity = parse_quantity_string("1.25Ki");
        assert!(quantity.is_ok());

        let quantity = quantity.unwrap().1;
        assert_eq!(quantity.value, Decimal::new(125, 2));
        assert_eq!(quantity.scale, Scale::Kilo);
        assert_eq!(quantity.format, Format::BinarySI);

        assert_eq!(quantity.to_string(), "1.25Ki".to_owned());
    }

    #[test]
    fn test_scientific_notation() {
        let quantity = parse_quantity_string("1.25e3");
        assert!(quantity.is_ok());

        let quantity = quantity.unwrap().1;
        assert_eq!(quantity.value, Decimal::new(1250, 0));
        assert_eq!(quantity.scale, Scale::One);
        // FIXME: This should probably be a decimal exponent format
        // but that would require rewriting the way it's handled in the parser
        // and for now this should be good enough
        assert_eq!(quantity.format, Format::DecimalSI);

        assert_eq!(quantity.to_string(), "1.25e3".to_owned());
    }

    #[test]
    fn test_decimal_notation() {
        let quantity = parse_quantity_string("1250000");
        assert!(quantity.is_ok());

        let quantity = quantity.unwrap().1;
        assert_eq!(quantity.value, Decimal::new(1250000, 0));
        assert_eq!(quantity.scale, Scale::One);
        assert_eq!(quantity.format, Format::DecimalSI);

        assert_eq!(quantity.to_string(), "1250000".to_owned());
    }

    #[test]
    fn test_incorrect_quantity() {
        let quantity = parse_quantity_string("1.25.123K");
        assert!(quantity.is_err());
    }

    #[test]
    fn test_zero_quantity() {
        let quantity = parse_quantity_string("0");
        assert!(quantity.is_ok());

        let quantity = quantity.unwrap().1;
        assert_eq!(quantity.value, Decimal::new(0, 0));
        assert_eq!(quantity.scale, Scale::One);
        assert_eq!(quantity.format, Format::DecimalSI);

        assert_eq!(quantity.to_string(), "0".to_owned());
    }

    #[test]
    fn test_milli_quantity() {
        let quantity = parse_quantity_string("100m");
        assert!(quantity.is_ok());

        let quantity = quantity.unwrap().1;
        assert_eq!(quantity.value, Decimal::new(100, 0));
        assert_eq!(quantity.scale, Scale::Milli);
        assert_eq!(quantity.format, Format::DecimalSI);

        assert_eq!(quantity.to_string(), "100m");
    }

    #[test]
    fn test_quantity_addition() {
        let q1 = parse_quantity_string("1Ki").unwrap().1;
        let q2 = parse_quantity_string("2Ki").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "3Ki");
    }
}
