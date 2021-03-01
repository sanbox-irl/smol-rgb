//! A smol library for simple (s)Rgb handling.
//!
//!

#![deny(missing_docs)]
#![no_std]

// keep the std when we're running tests
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;

/// A color used in blendable applications. On a technical level,
/// this color is in sRGB; however, this name is not very clear.
///
/// In code, we will say that this Color is `encoded`. This is generally the same
/// colorspace that texels in a texture are in. This color space is not valid
/// to perform mixing operations *between* colors in, so we must convert this
/// color space into a different color, [BlendableColor], with [to_blendable_color](Self::to_blendable_color)
/// before we do such operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncodedColor {
    /// The red component of the color.
    pub r: u8,

    /// The green component of the color.
    pub g: u8,

    /// The blue component of the color.
    pub b: u8,

    /// The alpha component of the color. This is the opacity of the color
    /// in most contexts, though can be used for nearly anything if necessary.
    pub a: u8,
}

impl EncodedColor {
    /// A basic white (255, 255, 255, 255) with full opacity.
    pub const WHITE: EncodedColor = EncodedColor::new(255, 255, 255, 255);

    /// A basic black (0, 0, 0, 255) with full opacity.
    pub const BLACK: EncodedColor = EncodedColor::new(0, 0, 0, 255);

    /// A black (0, 0, 0, 0) with zero opacity.
    pub const CLEAR: EncodedColor = EncodedColor::new(0, 0, 0, 0);

    /// Creates a new encoded 32bit color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Transforms this color into the Blendable color space.
    pub fn to_blendable(self) -> BlendableColor {
        BlendableColor {
            r: encoded_to_blendable(self.r),
            g: encoded_to_blendable(self.g),
            b: encoded_to_blendable(self.b),
            a: self.a as f32 / 255.0,
        }
    }

    /// Converts this color to an [f32; 4] array. This is **still in encoded
    /// space** but they are converted to an f32. This is mostly for compatability
    /// with other libraries which sometimes need to f32s even while in encoded sRGB.
    ///
    /// We use this dedicated function, rather than a `From` or `Into` because
    /// this is an unusual use of f32s, and in general, this module acts as if
    /// f32 == Blendable and u8 == Encoded, though this is not technically true.
    pub fn to_encoded_f32s(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Converts this color to an [f32; 4] array. This is **still in encoded
    /// space** but they are converted to an f32. This is mostly for compatability
    /// with other libraries which sometimes need to f32s even while in encoded sRGB.
    ///
    /// We use this dedicated function, rather than a `From` or `Into` because
    /// this is an unusual use of f32s, and in general, this module acts as if
    /// f32 == Blendable and u8 == Encoded, though this is not technically true.
    pub fn from_encoded_f32s(input: [f32; 4]) -> Self {
        Self::new(
            (input[0] * 255.0) as u8,
            (input[1] * 255.0) as u8,
            (input[2] * 255.0) as u8,
            (input[3] * 255.0) as u8,
        )
    }
}

impl fmt::Display for EncodedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encoded Color({}, {}, {}, {})",
            self.r, self.g, self.b, self.a
        )
    }
}

/// This is a Color in the `blendable` space. This represents
/// "linear sRGB". You should use this color space when blending colors on the CPU
/// or when sending uniforms to a blendable card.
///
/// Colors on disc are [Color], but to blend them correctly, you need to move them
/// into the `blendable` color space with [to_blendable_space](Color::to_blendable_space).
///
/// You *can* directly create this struct, but you probably don't want to. You'd need already
/// linear sRGB to correctly make this struct -- that's possible to have, but generally, textures,
/// color pickers (like photoshop), and outputted surface (like if you use a Color Picker on a game)
/// will all be in the encoded RGB space. Exceptions abound though, so it is possible to directly
/// create this color.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BlendableColor {
    /// The red component of the color.
    pub r: f32,

    /// The green component of the color.
    pub g: f32,

    /// The blue component of the color.
    pub b: f32,

    /// The alpha component of the color, normally the opacity in blending operations.
    pub a: f32,
}

impl BlendableColor {
    /// **You probably don't want to use this function.**
    /// This creates a color in the BlendableColor space directly. For this function to be valid,
    /// the colors given to this function **must be in the linear space already.**
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Transforms this color into the Encoded color space. Use this space to serialize
    /// colors.
    pub fn to_encoded_space(self) -> EncodedColor {
        EncodedColor {
            r: blendable_to_encoded(self.r),
            g: blendable_to_encoded(self.g),
            b: blendable_to_encoded(self.b),
            a: (self.a * 255.0) as u8,
        }
    }

    /// Creates an array representation of the color. This is useful for sending the color
    /// to a uniform, but is the same memory representation as `Self`.
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<[f32; 4]> for BlendableColor {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[f32; 4]> for BlendableColor {
    fn from(o: [f32; 4]) -> Self {
        Self::new(o[0], o[1], o[2], o[3])
    }
}

impl fmt::Display for BlendableColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

/// This function takes an encoded u8 and outputs a blendable space (linear) sRgb f32.
///
/// This is based on <https://bottosson.github.io/posts/colorwrong/> and similar
/// transfer functions.
pub fn encoded_to_blendable(input: u8) -> f32 {
    #[cfg(feature = "libm")]
    use libm::powf;

    #[cfg(feature = "std")]
    fn powf(f: f32, e: f32) -> f32 {
        f.powf(e)
    }

    let input = input as f32 / 255.0;

    if input >= 0.04045 {
        powf((input + 0.055) / 1.055, 2.4)
    } else {
        input / 12.92
    }
}

/// This function takes an blendable space f32 and outputs an encoded sRgb u8.
///
/// This is based on <https://bottosson.github.io/posts/colorwrong/> and similar
/// transfer functions.
pub fn blendable_to_encoded(input: f32) -> u8 {
    #[cfg(feature = "libm")]
    use libm::powf;

    #[cfg(feature = "std")]
    fn powf(f: f32, e: f32) -> f32 {
        f.powf(e)
    }

    let encoded_f32 = if input >= 0.0031308 {
        1.055 * powf(input, 1.0 / 2.4) - 0.055
    } else {
        12.92 * input
    };

    // this multiply to 256 is VERY odd! but otherwise,
    // 1.0 cannot translate to 1.0. Weirdly, this seems fine actually
    // in tests.
    (encoded_f32 * 256.0) as u8
}

impl From<BlendableColor> for EncodedColor {
    fn from(o: BlendableColor) -> Self {
        o.to_encoded_space()
    }
}

impl From<EncodedColor> for BlendableColor {
    fn from(o: EncodedColor) -> Self {
        o.to_blendable()
    }
}

#[cfg(feature = "bytemuck")]
impl BlendableColor {
    /// Creates an array representation of the color. This is useful for sending the color
    /// to a uniform, but is the same memory representation as `Self`.
    pub fn to_bits(self) -> [u8; 16] {
        bytemuck::cast(self.to_array())
    }
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for EncodedColor {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for EncodedColor {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for BlendableColor {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for BlendableColor {}

#[cfg(feature = "serde")]
const ENCODED_NAME: &str = "Encoded Color";

#[cfg(feature = "serde")]
impl serde::Serialize for EncodedColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTupleStruct;
        let mut seq = serializer.serialize_tuple_struct(ENCODED_NAME, 4)?;

        seq.serialize_field(&self.r)?;
        seq.serialize_field(&self.g)?;
        seq.serialize_field(&self.b)?;
        seq.serialize_field(&self.a)?;

        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for EncodedColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DeserializeColor;

        impl<'de> serde::de::Visitor<'de> for DeserializeColor {
            type Value = EncodedColor;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence of u8 colors")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let r = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                let g = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                let b = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                let a = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;

                Ok(EncodedColor { r, g, b, a })
            }
        }

        deserializer.deserialize_tuple_struct(ENCODED_NAME, 4, DeserializeColor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static_assertions::assert_eq_align!(EncodedColor, u8);
    static_assertions::assert_eq_size!(EncodedColor, [u8; 4]);

    #[test]
    fn encoding_decoding() {
        fn encode(input: u8, output: f32) {
            let o = encoded_to_blendable(input);
            std::println!("Expected {}, got {}", output, o);
            assert!((o - output).abs() < f32::EPSILON);
        }

        fn decode(input: f32, output: u8) {
            let o = blendable_to_encoded(input);
            assert_eq!(o, output);
        }

        encode(66, 0.05448028);
        encode(0, 0.0);
        encode(255, 1.0);
        encode(240, 0.8713672);
        encode(100, 0.1274377);
        encode(128, 0.2158605);

        decode(0.05448028, 66);
        decode(0.0, 0);
        decode(1.0, 255);
        decode(0.1274377, 100);
        decode(0.8713672, 240);
        decode(0.2158605, 128);
    }

    #[test]
    fn serde() {
        // json
        let color = EncodedColor::new(50, 50, 50, 255);
        let serialized = serde_json::to_string(&color).unwrap();
        assert_eq!("[50,50,50,255]", serialized);
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(color, deserialized);

        // yaml
        let serialized = serde_yaml::to_string(&color).unwrap();
        assert_eq!("---\n- 50\n- 50\n- 50\n- 255\n", serialized);
        let deserialized = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(color, deserialized);

        // more yaml (look I use serde_yaml)
        let start = "---\n- 22\n- 33\n- 100\n- 210";
        let color: EncodedColor = serde_yaml::from_str(start).unwrap();
        let base = EncodedColor::new(22, 33, 100, 210);
        assert_eq!(color, base);

        // bad serds
        let o = serde_yaml::from_str::<EncodedColor>("[0.2, 50, 50, 255]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedColor>("[20, 50, 50, 256]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedColor>("[20, 50, 245]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedColor>("[-20, 20, 50, 255]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedColor>("[20, 20, 50, 255, 255]");
        assert!(o.is_err());

        // and the big chungus, bincode
        let color = EncodedColor::new(44, 232, 8, 255);
        let buff = bincode::serialize(&color).unwrap();
        assert_eq!(buff, [44, 232, 8, 255]);

        let color = EncodedColor::new(200, 21, 22, 203);
        let buff = bincode::serialize(&color).unwrap();
        assert_eq!(buff, [200, 21, 22, 203]);
        let round_trip_color: EncodedColor = bincode::deserialize(&buff).unwrap();
        assert_eq!(color, round_trip_color);

        let buf = [14u8, 12, 3];
        let o = bincode::deserialize::<EncodedColor>(bytemuck::cast_slice(&buf));
        assert!(o.is_err());

        // okay and now with options, because otherwise it's hard to get errors
        // out of bincode...
        use bincode::Options;
        let deserialize = bincode::DefaultOptions::new();

        let buf = [14, 12];
        let o = deserialize.deserialize::<EncodedColor>(bytemuck::cast_slice(&buf));
        assert!(o.is_err());

        let buf = [14u64];
        let o = deserialize.deserialize::<EncodedColor>(bytemuck::cast_slice(&buf));
        assert!(o.is_err());

        let buf = [31.0f32];
        let o = deserialize.deserialize::<EncodedColor>(bytemuck::cast_slice(&buf));
        // lol, i don't like this. is there a way to make this not work? if you see this
        // and know the answer, please PR me!
        assert!(o.is_ok());
    }
}
