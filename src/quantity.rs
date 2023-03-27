use std::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt::Display,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use rust_decimal::prelude::*;

use crate::{format::Format, scale::Scale, utils::scale_format_to_string};

// - Parsed Quantity -

/// ParsedQuantity represents a parsed Kubernetes quantity.
///
/// ```rust
/// use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
/// use kube_quantity::{ParseQuantityError, ParsedQuantity};
///
/// // Kubernetes quantity
/// let k8s_quantity = Quantity("1Ki".to_string());
///
/// // Try parsing k8s quantity
/// let quantity: Result<ParsedQuantity, ParseQuantityError> = k8s_quantity.try_into();
///
/// assert_eq!(quantity.unwrap().to_string(), "1Ki");
/// ```
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

impl PartialEq for ParsedQuantity {
    fn eq(&self, other: &Self) -> bool {
        let mut lhs = self.clone();
        let mut rhs = other.clone();

        normalize_formats(&mut lhs, &mut rhs);
        normalize_scales(&mut lhs, &mut rhs);

        lhs.value.eq(&rhs.value)
    }
}

impl Eq for ParsedQuantity {}

impl PartialOrd for ParsedQuantity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParsedQuantity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut lhs = self.clone();
        let mut rhs = other.clone();

        normalize_formats(&mut lhs, &mut rhs);
        normalize_scales(&mut lhs, &mut rhs);

        lhs.value.cmp(&rhs.value)
    }
}

impl ParsedQuantity {
    /// Returns the value of the quantity as a string with the specified number of
    /// decimal points for fractional portion.
    /// Additionally it performs normalization, i.e., strips any trailing zero's from a value and converts -0 to 0.
    ///
    /// When a number is halfway between two others, it is rounded toward the
    /// nearest number that is away from zero. e.g. 6.4 -> 6, 6.5 -> 7, -6.5 -> -7
    ///
    /// ```rust
    /// use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
    /// use kube_quantity::ParsedQuantity;
    ///
    /// let k_quantity: ParsedQuantity = Quantity("1k".to_string()).try_into().unwrap();
    /// let ki_quantity: ParsedQuantity = Quantity("1Ki".to_string()).try_into().unwrap();
    ///
    /// let q3 = k_quantity + ki_quantity;
    ///
    /// assert_eq!(q3.to_string_with_precision(3), "2.024k");
    /// assert_eq!(q3.to_string_with_precision(2), "2.02k");
    /// assert_eq!(q3.to_string_with_precision(1), "2k");
    /// assert_eq!(q3.to_string_with_precision(0), "2k");
    /// ```
    pub fn to_string_with_precision(&self, precision: u32) -> String {
        format!(
            "{}{}",
            self.value
                .round_dp_with_strategy(precision, RoundingStrategy::MidpointAwayFromZero)
                .normalize(),
            scale_format_to_string(&self.scale, &self.format)
        )
    }

    /// Returns the value of the quantity as an f64.
    ///
    /// ```rust
    /// use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
    /// use kube_quantity::{ParseQuantityError, ParsedQuantity};
    ///
    /// // Kubernetes quantity
    /// let k8s_quantity = Quantity("1Ki".to_string());
    ///
    /// // Try parsing k8s quantity
    /// let quantity: Result<ParsedQuantity, ParseQuantityError> = k8s_quantity.try_into();
    ///
    /// assert_eq!(quantity.unwrap().to_bytes_f64(), Some(1024.0));
    /// ```
    pub fn to_bytes_f64(&self) -> Option<f64> {
        let scale: i32 = (&self.scale).into();

        self.value.to_f64().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_f64.powi(scale),
                    // Format::DecimalExponent => 1000_f64.powi(scale),
                    Format::DecimalSI => 1000_f64.powi(scale),
                }
        })
    }

    /// Returns the value of the quantity as an f32.
    pub fn to_bytes_f32(&self) -> Option<f32> {
        let scale: i32 = (&self.scale).into();

        self.value.to_f32().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_f32.powi(scale),
                    // Format::DecimalExponent => 1000_f32.powi(scale),
                    Format::DecimalSI => 1000_f32.powi(scale),
                }
        })
    }

    /// Returns the value of the quantity as an i128.
    pub fn to_bytes_i128(&self) -> Option<i128> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_i128().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_i128.pow(scale),
                    // Format::DecimalExponent => 1000_i128.pow(scale),
                    Format::DecimalSI => 1000_i128.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an i64.
    pub fn to_bytes_i64(&self) -> Option<i64> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_i64().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_i64.pow(scale),
                    // Format::DecimalExponent => 1000_i64.pow(scale),
                    Format::DecimalSI => 1000_i64.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an i32.
    pub fn to_bytes_i32(&self) -> Option<i32> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_i32().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_i32.pow(scale),
                    // Format::DecimalExponent => 1000_i32.pow(scale),
                    Format::DecimalSI => 1000_i32.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an i16.
    pub fn to_bytes_i16(&self) -> Option<i16> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_i16().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_i16.pow(scale),
                    // Format::DecimalExponent => 1000_i16.pow(scale),
                    Format::DecimalSI => 1000_i16.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an i8.
    /// This will only work if the scale is 0.
    pub fn to_bytes_i8(&self) -> Option<i8> {
        let scale: i32 = (&self.scale).into();

        if scale != 0 {
            return None;
        }

        self.value.to_i8()
    }

    /// Returns the value of the quantity as an isize.
    pub fn to_bytes_isize(&self) -> Option<isize> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_isize().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_isize.pow(scale),
                    // Format::DecimalExponent => 1000_isize.pow(scale),
                    Format::DecimalSI => 1000_isize.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an u128.
    pub fn to_bytes_u128(&self) -> Option<u128> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_u128().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_u128.pow(scale),
                    // Format::DecimalExponent => 1000_u128.pow(scale),
                    Format::DecimalSI => 1000_u128.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an u64.
    pub fn to_bytes_u64(&self) -> Option<u64> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_u64().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_u64.pow(scale),
                    // Format::DecimalExponent => 1000_u64.pow(scale),
                    Format::DecimalSI => 1000_u64.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an u32.
    pub fn to_bytes_u32(&self) -> Option<u32> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_u32().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_u32.pow(scale),
                    // Format::DecimalExponent => 1000_u32.pow(scale),
                    Format::DecimalSI => 1000_u32.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an u16.
    pub fn to_bytes_u16(&self) -> Option<u16> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_u16().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_u16.pow(scale),
                    // Format::DecimalExponent => 1000_u16.pow(scale),
                    Format::DecimalSI => 1000_u16.pow(scale),
                }
        })
    }

    /// Returns the value of the quantity as an u8.
    /// This will only work if the scale is 0.
    pub fn to_bytes_u8(&self) -> Option<u8> {
        let scale: i32 = (&self.scale).into();

        if scale != 0 {
            return None;
        }

        self.value.to_u8()
    }

    /// Returns the value of the quantity as an usize.
    pub fn to_bytes_usize(&self) -> Option<usize> {
        let scale: i32 = (&self.scale).into();
        let scale: u32 = scale.try_into().ok()?;

        self.value.to_usize().map(|value| {
            value
                * match &self.format {
                    Format::BinarySI => 1024_usize.pow(scale),
                    // Format::DecimalExponent => 1000_usize.pow(scale),
                    Format::DecimalSI => 1000_usize.pow(scale),
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
        //             Decimal::from_f32((1024_f32 / 1000_f32).pow(rhs.scale.clone().into()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_eq() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        assert_eq!(q1, q2);
    }

    #[test]
    fn test_partial_ne() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Mega,
            format: Format::DecimalSI,
        };

        assert_ne!(q1, q2);
    }

    #[test]
    fn test_ord_le() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };
        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Mega,
            format: Format::DecimalSI,
        };

        assert!(q1 < q2);
    }

    #[test]
    fn test_ord_leq() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::DecimalSI,
        };
        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::DecimalSI,
        };

        assert!(q1 <= q2);
    }

    #[test]
    fn test_ord_le_different_formats() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::One,
            format: Format::BinarySI,
        };
        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::One,
            format: Format::DecimalSI,
        };

        assert!(q1 <= q2);
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_eq_different_formats_and_scales() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1024.0).unwrap(),
            scale: Scale::One,
            format: Format::DecimalSI,
        };

        assert_eq!(q1, q2);
    }

    #[test]
    fn test_ord_gt() {
        let q1 = ParsedQuantity {
            value: Decimal::from_f32(1.0).unwrap(),
            scale: Scale::Kilo,
            format: Format::BinarySI,
        };

        let q2 = ParsedQuantity {
            value: Decimal::from_f32(1020.0).unwrap(),
            scale: Scale::One,
            format: Format::DecimalSI,
        };

        assert!(q1 > q2);
    }
}
