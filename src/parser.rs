use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

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
}

impl Display for ParsedQuantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_representation = format!(
            "{}{}",
            self.value,
            scale_format_to_string(&self.scale, &self.format)
        );

        write!(f, "{}", string_representation)
    }
}

// Standard operations on parsed quantities
impl Add for ParsedQuantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        let mut rhs = rhs;

        // Bring both quantities to the same format
        // - If the formats are different, use the lhs format as output format and
        //   multiply the rhs value by the format multiplier
        normalize_formats(&mut lhs, &mut rhs);

        // Bring both scales to the same ones
        // - If the scales are different, use the smaller scale as output scale
        normalize_scales(&mut lhs, &mut rhs);

        // Add the normalized values
        let value = lhs.value.add(rhs.value).normalize();

        Self {
            value,
            scale: lhs.scale,
            format: lhs.format,
        }
    }
}

impl Sub for ParsedQuantity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        let mut rhs = rhs;

        // Bring both quantities to the same format
        // - If the formats are different, use the lhs format as output format and
        //   multiply the rhs value by the format multiplier
        normalize_formats(&mut lhs, &mut rhs);

        // Bring both scales to the same ones
        // - If the scales are different, use the smaller scale as output scale
        normalize_scales(&mut lhs, &mut rhs);

        // Subtract the normalized values
        let value = lhs.value.sub(rhs.value).normalize();

        Self {
            value,
            scale: lhs.scale,
            format: lhs.format,
        }
    }
}

impl Neg for ParsedQuantity {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            value: self.value.neg(),
            scale: self.scale,
            format: self.format,
        }
    }
}

impl AddAssign for ParsedQuantity {
    fn add_assign(&mut self, rhs: Self) {
        let mut rhs = rhs;

        normalize_formats(self, &mut rhs);
        normalize_scales(self, &mut rhs);

        self.value.add_assign(rhs.value);
    }
}

impl SubAssign for ParsedQuantity {
    fn sub_assign(&mut self, rhs: Self) {
        let mut rhs = rhs;

        normalize_formats(self, &mut rhs);
        normalize_scales(self, &mut rhs);

        self.value.sub_assign(rhs.value);
    }
}

impl ParsedQuantity {
    /// Returns the value of the quantity as a string with a given precision after
    /// the decimal point.
    pub fn to_string_with_precision(&self, precision: u32) -> String {
        format!(
            "{}{}",
            self.value.round_dp(precision).normalize(),
            scale_format_to_string(&self.scale, &self.format)
        )
    }

    /// Returns the value of the quantity as an f64.
    pub fn to_bytes_f64(&self) -> Option<f64> {
        let scale: i32 = (&self.scale).into();

        self.value.to_f64().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_f64.powi(scale),
                    // Format::DecimalExponent => 1000_f64.powi(multiplier),
                    Format::DecimalSI => 1000_f64.powi(scale),
                }
        })
    }
}

fn normalize_scales(lhs: &mut ParsedQuantity, rhs: &mut ParsedQuantity) {
    let rhs_scale: i32 = (&rhs.scale).into();
    let lhs_scale: i32 = (&lhs.scale).into();
    let multiplier = rhs_scale.abs_diff(lhs_scale).to_i32().unwrap_or_default();

    match lhs_scale.cmp(&rhs_scale) {
        std::cmp::Ordering::Less => {
            // Bring the rhs to the lower scale (lhs)
            rhs.value = rhs.value
                * Decimal::from_f32(match &rhs.format {
                    Format::BinarySI => 1024_f32.powi(multiplier),
                    // Format::DecimalExponent => 1000_f32.powi(multiplier),
                    Format::DecimalSI => 1000_f32.powi(multiplier),
                })
                .unwrap_or_default();
            rhs.scale = lhs.scale.clone();
        }
        std::cmp::Ordering::Equal => {
            // If equal do nothing
        }
        std::cmp::Ordering::Greater => {
            // Bring the lhs to the lower scale (rhs)
            lhs.value = lhs.value
                * Decimal::from_f32(match &lhs.format {
                    Format::BinarySI => 1024_f32.powi(multiplier),
                    // Format::DecimalExponent => 1000_f32.powi(multiplier),
                    Format::DecimalSI => 1000_f32.powi(multiplier),
                })
                .unwrap_or_default();
            lhs.scale = rhs.scale.clone();
        }
    }
}

fn normalize_formats(lhs: &mut ParsedQuantity, rhs: &mut ParsedQuantity) {
    match (&lhs.format, &rhs.format) {
        (Format::BinarySI, Format::BinarySI) => {}
        // (Format::BinarySI, Format::DecimalExponent) => {
        //     let value = (rhs.value)
        //         .mul(
        //             Decimal::from_f32((1024_f32 / 1000_f32).powi(rhs.scale.clone().into()))
        //                 .unwrap_or_default()
        //                 .normalize(),
        //         )
        //         .normalize();

        //     rhs.value = value;
        //     rhs.format = Format::BinarySI;
        // }
        (Format::BinarySI, Format::DecimalSI) => {
            let value = rhs
                .value
                .mul(
                    Decimal::from_f32((1000_f32 / 1024_f32).powi(rhs.scale.clone().into()))
                        .unwrap_or_default()
                        .normalize(),
                )
                .normalize();

            rhs.value = value;
            rhs.format = Format::BinarySI;
        }
        // (Format::DecimalExponent, Format::BinarySI) => todo!(),
        // (Format::DecimalExponent, Format::DecimalExponent) => {}
        // (Format::DecimalExponent, Format::DecimalSI) => todo!(),
        (Format::DecimalSI, Format::BinarySI) => {
            let value = rhs
                .value
                .mul(
                    Decimal::from_f32((1024_f32 / 1000_f32).powi(rhs.scale.clone().into()))
                        .unwrap_or_default()
                        .normalize(),
                )
                .normalize();

            rhs.value = value;
            rhs.format = Format::DecimalSI;
        }
        // (Format::DecimalSI, Format::DecimalExponent) => {
        //     rhs.format = Format::DecimalSI;
        // }
        (Format::DecimalSI, Format::DecimalSI) => {}
    };
}

// - Format -

#[derive(Debug, Clone, PartialEq, Eq)]
enum Format {
    /// e.g., 12Mi = (12 * 2^20) = (12 * 1024^2)
    BinarySI,
    // /// e.g., 12e6 = (12 * 10^6)
    // DecimalExponent,
    /// e.g., 12M = (12 * 10^6) = (12 * 1000^2)
    DecimalSI,
}

// - Scale -

/// Scale is used for getting and setting the base-10 scaled value. Base-2
/// scales are omitted for mathematical simplicity.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default)]
enum Scale {
    Milli,
    #[default]
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
impl From<Scale> for i32 {
    fn from(value: Scale) -> Self {
        (&value).into()
    }
}

impl From<&Scale> for i32 {
    fn from(value: &Scale) -> Self {
        // https://en.wikipedia.org/wiki/Kilobyte
        match value {
            Scale::Milli => -1,
            Scale::One => 0,
            Scale::Kilo => 1,
            Scale::Mega => 2,
            Scale::Giga => 3,
            Scale::Tera => 4,
            Scale::Peta => 5,
            Scale::Exa => 6,
        }
    }
}

impl TryFrom<i32> for Scale {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Scale::Milli),
            0 => Ok(Scale::One),
            1 => Ok(Scale::Kilo),
            2 => Ok(Scale::Mega),
            3 => Ok(Scale::Giga),
            4 => Ok(Scale::Tera),
            5 => Ok(Scale::Peta),
            6 => Ok(Scale::Exa),
            _ => Err(()),
        }
    }
}

// --- Functions ---

/// Returns the string representation of the scale and format
fn scale_format_to_string(scale: &Scale, format: &Format) -> String {
    match format {
        Format::BinarySI => match scale {
            Scale::Milli => "".to_owned(),
            Scale::One => "".to_owned(),
            Scale::Kilo => "Ki".to_owned(),
            Scale::Mega => "Mi".to_owned(),
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
        // Format::DecimalExponent => "e".to_owned(),
    }
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
