//! Two typefaces the backend has never heard of.
//!
//! This is the point of a registry rather than a fixed set: the backend ships
//! one family, and anything that wants another brings it. A firmware for a
//! board with room in flash adds three; a firmware for a board without adds
//! none and pays for none.
//!
//! Both are assembled with [`font_tier!`], which is the same macro the
//! backend builds Helvetica with — there is no privileged path.
//!
//! # The cost, measured
//!
//! Each family here is twelve faces of bitmaps in `.rodata`, and they are not
//! cheap. Built for a Badger 2040:
//!
//! | what [`FAMILIES`] holds | firmware |
//! |---|---|
//! | `&[&HELVETICA]` — the backend's own, alone | 103 KB |
//! | all three | 206 KB |
//!
//! **The two extra families double the firmware.** That is 5% of a 2MB
//! RP2040's flash and perfectly affordable there; it would be impossible on a
//! part with 256K. This is exactly why the backend ships one family and takes
//! whatever else it is given: a board that cannot spare the space shortens
//! [`FAMILIES`] and pays for nothing it does not use — the two below are
//! reachable from nowhere else, so dropping them from that list drops their
//! bitmaps from the binary.

use xpui_eg::{Family, HELVETICA, Tier, font_tier, u8g2};

/// Courier — a fixed-pitch serif, and about as far from Helvetica as this
/// collection goes.
///
/// The declared bands are the **taller** of each pair: this family's bold runs
/// one to two pixels above its regular, which Helvetica's does not. Getting
/// that wrong puts the extra rows of a bold heading into the row below it.
const COUR_08: Tier = font_tier!(12, u8g2::u8g2_font_courR08_tf, u8g2::u8g2_font_courB08_tf);
const COUR_10: Tier = font_tier!(17, u8g2::u8g2_font_courR10_tf, u8g2::u8g2_font_courB10_tf);
const COUR_12: Tier = font_tier!(18, u8g2::u8g2_font_courR12_tf, u8g2::u8g2_font_courB12_tf);
const COUR_14: Tier = font_tier!(21, u8g2::u8g2_font_courR14_tf, u8g2::u8g2_font_courB14_tf);
const COUR_18: Tier = font_tier!(27, u8g2::u8g2_font_courR18_tf, u8g2::u8g2_font_courB18_tf);
const COUR_24: Tier = font_tier!(34, u8g2::u8g2_font_courR24_tf, u8g2::u8g2_font_courB24_tf);

pub static COURIER: Family = Family {
    name: "Courier",
    tiers: &[COUR_08, COUR_10, COUR_12, COUR_14, COUR_18, COUR_24],
    fallback: Some(&HELVETICA),
};

/// New Century Schoolbook — a book serif, cut for reading rather than for
/// labels.
const NCEN_08: Tier = font_tier!(14, u8g2::u8g2_font_ncenR08_tf, u8g2::u8g2_font_ncenB08_tf);
const NCEN_10: Tier = font_tier!(19, u8g2::u8g2_font_ncenR10_tf, u8g2::u8g2_font_ncenB10_tf);
const NCEN_12: Tier = font_tier!(20, u8g2::u8g2_font_ncenR12_tf, u8g2::u8g2_font_ncenB12_tf);
const NCEN_14: Tier = font_tier!(24, u8g2::u8g2_font_ncenR14_tf, u8g2::u8g2_font_ncenB14_tf);
const NCEN_18: Tier = font_tier!(30, u8g2::u8g2_font_ncenR18_tf, u8g2::u8g2_font_ncenB18_tf);
const NCEN_24: Tier = font_tier!(41, u8g2::u8g2_font_ncenR24_tf, u8g2::u8g2_font_ncenB24_tf);

pub static CENTURY: Family = Family {
    name: "Century",
    tiers: &[NCEN_08, NCEN_10, NCEN_12, NCEN_14, NCEN_18, NCEN_24],
    fallback: Some(&HELVETICA),
};

/// What the gallery offers, in the order a picker lists them.
///
/// Helvetica first because it is what everything opens in.
pub static FAMILIES: &[&Family] = &[&HELVETICA, &COURIER, &CENTURY];
