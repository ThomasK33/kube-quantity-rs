/// Scale is used for getting and setting the base-10 scaled value. Base-2
/// scales are omitted for mathematical simplicity.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default)]
pub(crate) enum Scale {
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
