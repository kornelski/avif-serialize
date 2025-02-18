#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ColorPrimaries {
    /// ITU-R BT1361
    Bt709 = 1,
    Unspecified = 2,
    /// ITU-R BT601-6 525
    Bt601 = 6,
    /// ITU-R BT2020
    Bt2020 = 9,
    /// SMPTE ST 431-2
    DciP3 = 11,
    /// SMPTE ST 432-1
    DisplayP3 = 12,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransferCharacteristics {
    /// ITU-R BT1361
    Bt709 = 1,
    Unspecified = 2,
    /// BT.470 System M (historical)
    Bt470M = 4,
    /// BT.470 System B, G (historical)
    Bt470BG,
    /// ITU-R BT601-6 525
    Bt601 = 6,
    /// SMPTE 240 M
    Smpte240,
    /// "Linear transfer characteristics"
    Linear = 8,
    /// "Logarithmic transfer characteristic (100:1 range)"
    Log = 9,
    /// "Logarithmic transfer characteristic (100 * Sqrt(10) : 1 range)"
    LogSqrt = 10,
    /// IEC 61966-2-4
    Iec61966,
    /// BT.1361 extended color gamut system (historical)
    Bt1361,
    /// sRGB
    Srgb = 13,
    /// ITU-R BT2020 for 10-bit system
    Bt2020_10 = 14,
    /// ITU-R BT2020 for 12-bit system
    Bt2020_12 = 15,
    /// SMPTE ST 2084, ITU BT.2100 PQ
    Smpte2084,
    /// SMPTE ST 428
    Smpte428,
    /// BT.2100 HLG (Hybrid Log Gamma), ARIB STD-B67
    Hlg,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MatrixCoefficients {
    /// GBR (sRGB)
    Rgb = 0,
    /// ITU-R BT1361
    Bt709 = 1,
    Unspecified = 2,
    /// ITU-R BT601-6 525
    Bt601 = 6,
    Ycgco = 8,
    /// ITU-R BT2020 non-constant luminance system
    Bt2020Ncl = 9,
    /// ITU-R BT2020 constant luminance system
    Bt2020Cl = 10,
}
