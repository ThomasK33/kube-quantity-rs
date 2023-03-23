use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use rust_decimal::prelude::*;

use crate::{format::Format, scale::Scale, utils::scale_format_to_string};

// - Parsed Quantity -

#[derive(Debug, Clone)]
pub struct ParsedQuantity {
    // The actual value of the quantity
    pub(crate) value: Decimal,
    // Scale used to indicate the base-10 exponent of the value
    pub(super) scale: Scale,
    // Used to indicate the format of the suffix used
    pub(super) format: Format,
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
            rhs.value *= Decimal::from_f32(match &rhs.format {
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
            lhs.value *= Decimal::from_f32(match &lhs.format {
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
