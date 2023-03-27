// - Format -

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Format {
    /// e.g., 12Mi = (12 * 2^20) = (12 * 1024^2)
    #[default]
    BinarySI,
    // /// e.g., 12e6 = (12 * 10^6)
    // DecimalExponent,
    /// e.g., 12M = (12 * 10^6) = (12 * 1000^2)
    DecimalSI,
}
