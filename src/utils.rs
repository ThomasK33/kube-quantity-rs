use crate::{format::Format, scale::Scale};

/// Returns the string representation of the scale and format
pub(crate) fn scale_format_to_string(scale: &Scale, format: &Format) -> String {
    match format {
        Format::BinarySI => match scale {
            Scale::Nano => "n".to_owned(),
            Scale::Micro => "u".to_owned(),
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
            Scale::Nano => "n".to_owned(),
            Scale::Micro => "u".to_owned(),
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
