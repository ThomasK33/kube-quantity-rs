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

use crate::{format::Format, quantity::ParsedQuantity, scale::Scale};

// --- Errors ---

#[derive(Debug, Error)]
pub enum ParseQuantityError {
    /// The string is empty
    #[error("empty string")]
    EmptyString,

    /// The string is not a valid quantity format
    #[error("quantity parsing failed")]
    ParsingFailed(#[from] nom::Err<nom::error::Error<String>>),

    /// The numeric value is not a valid decimal number
    #[error("decimal parsing failed")]
    DecimalParsingFailed,
}

// --- Parsers ---

/// Parses a signed number from a string and returns the remaining input and the
/// parsed quantity
pub(crate) fn parse_quantity_string(
    input: &str,
) -> Result<(&str, ParsedQuantity), ParseQuantityError> {
    if input.is_empty() {
        return Err(ParseQuantityError::EmptyString);
    }

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
            value: Decimal::from_f64(signed_number)
                .ok_or(ParseQuantityError::DecimalParsingFailed)?,
        },
    ))
}

/// Parses a signed number from a string and returns the remaining input and the
/// signed number
fn parse_signed_number(input: &str) -> IResult<&str, f64> {
    // Default to true
    let (input, positive) =
        opt(parse_sign)(input).map(|(input, positive)| (input, positive.unwrap_or(true)))?;
    // Default num to 0.0
    let (input, num) = opt(double)(input).map(|(input, num)| (input, num.unwrap_or(0.0)))?;

    Ok((input, if positive { num } else { -num }))
}

/// Parses the suffix and returns the remaining input and the format and scale
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

/// Parses a sign from a string and returns the remaining input and the sign
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

        // assert_eq!(quantity.to_string(), "1.25e3".to_owned());
        assert_eq!(quantity.to_string(), "1250".to_owned());
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
    fn test_quantity_addition_binary_si() {
        let q1 = parse_quantity_string("1Ki").unwrap().1;
        let q2 = parse_quantity_string("2Ki").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "3Ki");
    }

    #[test]
    fn test_quantity_addition_binary_si_2() {
        let q1 = parse_quantity_string("1.5Mi").unwrap().1;
        let q2 = parse_quantity_string("2Mi").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "3.5Mi");
    }

    #[test]
    fn test_quantity_addition_binary_si_mixed_scales() {
        let q1 = parse_quantity_string("1Ki").unwrap().1;
        let q2 = parse_quantity_string("2Mi").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "2049Ki");
    }

    #[test]
    fn test_quantity_addition_binary_si_decimal_exponent() {
        let q1 = parse_quantity_string("12Mi").unwrap().1;
        let q2 = parse_quantity_string("12e6").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "24582912");
    }

    #[test]
    fn test_quantity_addition_binary_si_decimal_si() {
        let q1 = parse_quantity_string("12Mi").unwrap().1;
        let q2 = parse_quantity_string("12M").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "23.4440916Mi");
    }

    #[test]
    fn test_quantity_addition_binary_si_decimal_si_2() {
        let q1 = parse_quantity_string("12Ki").unwrap().1;
        let q2 = parse_quantity_string("1000").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "13288");
    }

    #[test]
    fn test_quantity_addition_decimal_si_milli_addition() {
        let q1 = parse_quantity_string("100m").unwrap().1;
        let q2 = parse_quantity_string("200m").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "300m");
    }

    #[test]
    fn test_quantity_addition_decimal_si_milli_addition_2() {
        let q1 = parse_quantity_string("100m").unwrap().1;
        let q2 = parse_quantity_string("1").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "1100m");
    }

    #[test]
    fn test_quantity_addition_decimal_exponent() {
        let q1 = parse_quantity_string("10e3").unwrap().1;
        let q2 = parse_quantity_string("10e3").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "20000");
    }

    #[test]
    fn test_quantity_addition_decimal_exponent_2() {
        let q1 = parse_quantity_string("10e4").unwrap().1;
        let q2 = parse_quantity_string("10e3").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "110000");
    }

    #[test]
    fn test_quantity_addition_decimal_exponent_binary_si() {
        let q1 = parse_quantity_string("10e3").unwrap().1;
        let q2 = parse_quantity_string("1Ki").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string_with_precision(0), "11024");
    }

    #[test]
    fn test_quantity_addition_decimal_exponent_decimal_si() {
        let q1 = parse_quantity_string("10e3").unwrap().1;
        let q2 = parse_quantity_string("1k").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "11000");
    }

    #[test]
    fn test_quantity_addition_decimal_exponent_decimal_si_2() {
        let q1 = parse_quantity_string("10e2").unwrap().1;
        let q2 = parse_quantity_string("1k").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "2000");
    }

    #[test]
    fn test_quantity_addition_decimal_si() {
        let q1 = parse_quantity_string("1M").unwrap().1;
        let q2 = parse_quantity_string("2M").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "3M");
    }

    #[test]
    fn test_quantity_addition_decimal_si_2() {
        let q1 = parse_quantity_string("1k").unwrap().1;
        let q2 = parse_quantity_string("2M").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "2001k");
    }

    #[test]
    fn test_quantity_addition_decimal_si_binary_si() {
        let q1 = parse_quantity_string("1k").unwrap().1;
        let q2 = parse_quantity_string("1Ki").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string_with_precision(3), "2.024k");
    }

    #[test]
    fn test_quantity_addition_decimal_si_binary_si_2() {
        let q1 = parse_quantity_string("1M").unwrap().1;
        let q2 = parse_quantity_string("1Mi").unwrap().1;

        let q3 = q1 + q2;

        assert_eq!(q3.to_string(), "2.0485761M");
    }

    #[test]
    fn test_quantity_subtraction() {
        let q1 = parse_quantity_string("5Mi").unwrap().1;
        let q2 = parse_quantity_string("2Mi").unwrap().1;

        let q3 = q1 - q2;

        assert_eq!(q3.to_string(), "3Mi");
    }

    #[test]
    fn test_quantity_add_assign() {
        let mut q1 = parse_quantity_string("5Mi").unwrap().1;
        let q2 = parse_quantity_string("2Mi").unwrap().1;

        q1 += q2;

        assert_eq!(q1.to_string(), "7Mi");
    }

    #[test]
    fn test_quantity_sub_assign() {
        let mut q1 = parse_quantity_string("5Mi").unwrap().1;
        let q2 = parse_quantity_string("2Mi").unwrap().1;

        q1 -= q2;

        assert_eq!(q1.to_string(), "3Mi");
    }

    #[test]
    fn test_quantity_sub_mixed() {
        let q1 = parse_quantity_string("2M").unwrap().1;
        let q2 = parse_quantity_string("500k").unwrap().1;

        let q3 = q1 - q2;

        assert_eq!(q3.to_string(), "1500k");
    }
}
