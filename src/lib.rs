//! A smol library for (s)Rgb color handling.
//!
//! # Quick-start
//!
//! To use this in your project, add this to your Cargo.toml:
//!
//! ```toml
//! smol-rgb = "0.1.0"
//! ```
//!
//! no-std is supported, but requires `libm` to work, like so:
//!
//! ```toml
//! smol-rgb = { version = "0.1.0", default-features = false, features = ["libm"]  }
//! ```
//!
//! We also support two other features: `serde` and `bytemuck`. `serde` support works
//! across a variety of backends such as yaml, json, and bincode.
//!
//! # Who is this library for?
//!
//! This library is designed for the programmer who:
//!  - is working with graphics on the GPU (such as games)
//!  - works entirely or almost entirely with sRGB (if you don't know what that means, that's probably you),
//!  - and doesn't care about color beyond it "just working" correctly.
//!
//! This library can also serve as a good starting point to learn more complex color theory.
//!
//! For users who are comfortable working in color spaces, you should check out the much more
//! complicated library [palette](https://github.com/Ogeon/palette). It is significantly
//! more complicated, but also equally more capable.
//!
//! This library, on the other hand, only works with sRGB, and is designed only to help the programmer
//! work with sRGB in a simple manner.
//!
//! # It's not always RGB, but we can make it only sRGB.
//!
//! Textures, color pickers (like egui or imgui's pickers) are generally in "encoded" sRGB.
//! In this library, that means 4 u8s, each of which describe how much `r`, `g`, `b`, and `a`
//! should be in an image. On a very technical level, this is a specification called
//! IEC 61966-2-1:1999, but you should never remember this again. In this library, this space is
//! called `EncodedRgb`. If you use photoshop and use the color picker on a color (generally),
//! the number you get out is going to be in encoded sRGB, which this library handles in EncodedRgb.
//! That "generally" might have worried you; unless you know you did something odd, however, it shouldn't.
//! If you're authoring texture in Photoshop or in Aseprite, you'll be working in sRGB (unless you make
//! it so you aren't, but don't do that).
//!
//! Encoded sRGB is just the bee's knees, except that it's basically useless to *do* things in.
//! When you want to *blend* colors (add them, multiply them, basically do anything to them),
//! you need to convert those colors into "linear" space. In this library, we call this `LinearColor`.
//! Whereas `EncodedRgb` is just 4 u8s, `LinearColor` is 4 f32s, each of which has been transferred
//! from "encoded" space to "linear" space. The more complete terms would be that they have been
//! transferred from "encoded sRGB" to "linear sRGB", but don't think about it too much -- basically,
//! now they're in a place where they can be mixed with each other.
//!
//! # When does this happen Magically?
//!
//! Most of the time, in your library or application, your colors will be in `EncodedRgb`
//! and you won't think much about it. If you use a tool like egui or imgui-rs, you'll set colors
//! from those color picker applets directly into your `EncodedRgb` and call it a day.
//! In fact, if you're working in something like Opengl or Vulkan, and you're passing in Colors
//! in a Vertex Attribute, you may *still* use `EncodedRgb` in that circumstance, so long as
//! you make sure to make that attribute normalized correctly (in [vulkan](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkFormat.html),
//! and in [opengl](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glVertexAttribPointer.xhtml)).
//!
//! And of course, I've said a few times now that Textures are in EncodedRgb, yet, of course,
//! when you access them in a Shader, you can tint them with uniforms easily and correctly,
//! so they must also be in linear at that stage, right?
//!
//! The answer is yes! The GPU, when it samples a texture, will convert it into LinearRgb *for you.*
//! It will also, if you've set up your vertex attributes like above, do the same for those.
//!
//! Even more confusingly, after your fragment shader is done working in linear colors, it will (generally)
//! be converted *back* into EncodedRgb for final output. This is why if you use a color picker on your screen,
//! you'll still be getting EncodedRgb colors out! If your monitor itself is in sRgb (and many are), then you'll
//! even be displaying those colors in EncodedRgb.
//!
//! # When do I need to transfer EncodedRgb to LinearRgb myself?
//!
//! In two circumstances, for most programmers -- when you're blending colors yourself on the CPU, or when
//! you're sending a color to a uniform to be blended with another LinearRgb color (like a sampled texture) on the GPU.
//!
//! You might think to yourself that you commonly sent colors before you read this in "what you're calling 'EncodedRgb'" and
//! it worked out just fine. That's probably true! Almost all games have some color error, because it's just so easy to do
//! accidentally. However, I might point out that probably you or an artist just fiddled with the encoded color until it
//! mixed correctly, so it looked more or less right on the GPU. Or perhaps there was some other weirdness going on!
//!
//! # A quick final note on alpha
//!
//! This library uses the term `Rgb` for its color space, which is really `sRgb` with an `alpha` channel.
//! We do this for the sake of simplicity -- alpha is almost always desired in grapics applications, and like, come on,
//! you can spare the byte.
//!
//! If this library picks up enough traction, users might want to split it into `Rgb` and `Rgba`. Leave an issue
//! if that's desired.

#![deny(missing_docs, broken_intra_doc_links)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;

/// A color used in linear applications. On a technical level,
/// this color is in sRGB; however, this name is not very clear.
///
/// In code, we will say that this Color is `encoded`. This is generally the same
/// colorspace that texels in a texture are in. This color space is not valid
/// to perform mixing operations *between* colors in, so we must convert this
/// color space into a different color, [LinearRgb], with [to_linear](Self::to_linear)
/// before we do such operations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncodedRgb {
    /// The red component of the color.
    pub r: u8,

    /// The green component of the color.
    pub g: u8,

    /// The blue component of the color.
    pub b: u8,

    /// The alpha component of the color, normally the opacity in blending operations.
    pub a: u8,
}

impl EncodedRgb {
    /// A basic white (255, 255, 255, 255) with full opacity.
    pub const WHITE: EncodedRgb = EncodedRgb::new(255, 255, 255, 255);

    /// A basic black (0, 0, 0, 255) with full opacity.
    pub const BLACK: EncodedRgb = EncodedRgb::new(0, 0, 0, 255);

    /// A black (0, 0, 0, 0) with zero opacity.
    pub const CLEAR: EncodedRgb = EncodedRgb::new(0, 0, 0, 0);

    /// Creates a new encoded 32bit color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Transforms this color into the Linear color space.
    pub fn to_linear(self) -> LinearRgb {
        LinearRgb {
            r: encoded_to_linear(self.r),
            g: encoded_to_linear(self.g),
            b: encoded_to_linear(self.b),
            a: self.a as f32 / 255.0,
        }
    }

    /// Converts this color to an [f32; 4] array. This is **still in encoded
    /// space** but they are converted to an f32. This is mostly for compatability
    /// with other libraries which sometimes need to f32s even while in encoded sRGB.
    ///
    /// We use this dedicated function, rather than a `From` or `Into` because
    /// this is an unusual use of f32s, and in general, this module acts as if
    /// f32 == Linear and u8 == Encoded, though this is not technically true.
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
    /// f32 == Linear and u8 == Encoded, though this is not technically true.
    pub fn from_encoded_f32s(input: [f32; 4]) -> Self {
        Self::new(
            (input[0] * 255.0) as u8,
            (input[1] * 255.0) as u8,
            (input[2] * 255.0) as u8,
            (input[3] * 255.0) as u8,
        )
    }
}

impl From<(u8, u8, u8, u8)> for EncodedRgb {
    fn from(o: (u8, u8, u8, u8)) -> Self {
        Self {
            r: o.0,
            g: o.1,
            b: o.2,
            a: o.3,
        }
    }
}

impl Into<(u8, u8, u8, u8)> for EncodedRgb {
    fn into(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}

impl fmt::Debug for EncodedRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EncodedRgb")
            .field(&self.r)
            .field(&self.g)
            .field(&self.b)
            .field(&self.a)
            .finish()
    }
}

impl fmt::Display for EncodedRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "r: {}, g: {}, b: {}, a: {}, {:x}{:x}{:x}{:x}",
            self.r, self.g, self.b, self.a, self.r, self.g, self.b, self.a
        )
    }
}

/// This is a Color in the `linear` space. This represents
/// "linear sRGB". You should use this color space when blending colors on the CPU
/// or when sending uniforms to a linear card.
///
/// Colors on disc are [EncodedRgb], but to blend them correctly, you need to move them
/// into the `linear` color space with [to_linear](EncodedRgb::to_linear).
///
/// You *can* directly create this struct, but you probably don't want to. You'd need already
/// linear sRGB to correctly make this struct -- that's possible to have, but generally, textures,
/// color pickers (like photoshop), and outputted surface (like if you use a Color Picker on a game)
/// will all be in the encoded RGB space. Exceptions abound though, so it is possible to directly
/// create this color.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct LinearRgb {
    /// The red component of the color.
    pub r: f32,

    /// The green component of the color.
    pub g: f32,

    /// The blue component of the color.
    pub b: f32,

    /// The alpha component of the color, normally the opacity in blending operations.
    pub a: f32,
}

impl LinearRgb {
    /// **You probably don't want to use this function.**
    /// This creates a color in the LinearColor space directly. For this function to be valid,
    /// the colors given to this function **must be in the linear space already.**
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Transforms this color into the Encoded color space. Use this space to serialize
    /// colors.
    pub fn to_encoded_space(self) -> EncodedRgb {
        EncodedRgb {
            r: linear_to_encoded(self.r),
            g: linear_to_encoded(self.g),
            b: linear_to_encoded(self.b),
            a: (self.a * 255.0) as u8,
        }
    }

    /// Creates an array representation of the color. This is useful for sending the color
    /// to a uniform, but is the same memory representation as `Self`. [LinearRgb] also implements
    /// Into, but this function is often more convenient.
    pub fn to_array(self) -> [f32; 4] {
        self.into()
    }

    /// Encodes the 4 floats as 16 u8s. This is useful for sending the color
    /// to a uniform, but is the same memory representation as `Self` -- ie,
    /// the bits have just been reinterpreted as 16 u8s, but they're still secret floats.
    pub fn to_bits(self) -> [u8; 16] {
        unsafe { core::mem::transmute(self.to_array()) }
    }
}

impl Into<[f32; 4]> for LinearRgb {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[f32; 4]> for LinearRgb {
    fn from(o: [f32; 4]) -> Self {
        Self::new(o[0], o[1], o[2], o[3])
    }
}

impl Into<(f32, f32, f32, f32)> for LinearRgb {
    fn into(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

impl From<(f32, f32, f32, f32)> for LinearRgb {
    fn from(o: (f32, f32, f32, f32)) -> Self {
        Self::new(o.0, o.1, o.2, o.3)
    }
}

impl fmt::Debug for LinearRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LinearRgb")
            .field(&self.r)
            .field(&self.g)
            .field(&self.b)
            .field(&self.a)
            .finish()
    }
}

impl fmt::Display for LinearRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "r: {}, g: {}, b: {}, a: {}",
            self.r, self.g, self.b, self.a
        )
    }
}

/// This function takes an encoded u8 and outputs a linear space (linear) sRgb f32.
///
/// This is based on <https://bottosson.github.io/posts/colorwrong/> and similar
/// transfer functions.
pub fn encoded_to_linear(input: u8) -> f32 {
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

/// This function takes an linear space f32 and outputs an encoded sRgb u8.
///
/// This is based on <https://bottosson.github.io/posts/colorwrong/> and similar
/// transfer functions.
pub fn linear_to_encoded(input: f32) -> u8 {
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

impl From<LinearRgb> for EncodedRgb {
    fn from(o: LinearRgb) -> Self {
        o.to_encoded_space()
    }
}

impl From<EncodedRgb> for LinearRgb {
    fn from(o: EncodedRgb) -> Self {
        o.to_linear()
    }
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for EncodedRgb {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for EncodedRgb {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for LinearRgb {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for LinearRgb {}

#[cfg(feature = "serde")]
const ENCODED_NAME: &str = "Encoded Rgb";

#[cfg(feature = "serde")]
impl serde::Serialize for EncodedRgb {
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
impl<'de> serde::Deserialize<'de> for EncodedRgb {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DeserializeColor;

        impl<'de> serde::de::Visitor<'de> for DeserializeColor {
            type Value = EncodedRgb;

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

                Ok(EncodedRgb { r, g, b, a })
            }
        }

        deserializer.deserialize_tuple_struct(ENCODED_NAME, 4, DeserializeColor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static_assertions::assert_eq_align!(EncodedRgb, u8);
    static_assertions::assert_eq_size!(EncodedRgb, [u8; 4]);

    #[test]
    fn encoding_decoding() {
        fn encode(input: u8, output: f32) {
            let o = encoded_to_linear(input);
            std::println!("Expected {}, got {}", output, o);
            assert!((o - output).abs() < f32::EPSILON);
        }

        fn decode(input: f32, output: u8) {
            let o = linear_to_encoded(input);
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
        let color = EncodedRgb::new(50, 50, 50, 255);
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
        let color: EncodedRgb = serde_yaml::from_str(start).unwrap();
        let base = EncodedRgb::new(22, 33, 100, 210);
        assert_eq!(color, base);

        // bad serds
        let o = serde_yaml::from_str::<EncodedRgb>("[0.2, 50, 50, 255]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedRgb>("[20, 50, 50, 256]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedRgb>("[20, 50, 245]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedRgb>("[-20, 20, 50, 255]");
        assert!(o.is_err());
        let o = serde_yaml::from_str::<EncodedRgb>("[20, 20, 50, 255, 255]");
        assert!(o.is_err());

        // and the big chungus, bincode
        let color = EncodedRgb::new(44, 232, 8, 255);
        let buff = bincode::serialize(&color).unwrap();
        assert_eq!(buff, [44, 232, 8, 255]);

        let color = EncodedRgb::new(200, 21, 22, 203);
        let buff = bincode::serialize(&color).unwrap();
        assert_eq!(buff, [200, 21, 22, 203]);
        let round_trip_color: EncodedRgb = bincode::deserialize(&buff).unwrap();
        assert_eq!(color, round_trip_color);

        let buf = [14u8, 12, 3];
        let o = bincode::deserialize::<EncodedRgb>(bytemuck::cast_slice(&buf));
        assert!(o.is_err());

        // okay and now with options, because otherwise it's hard to get errors
        // out of bincode...
        use bincode::Options;
        let deserialize = bincode::DefaultOptions::new();

        let buf = [14, 12];
        let o = deserialize.deserialize::<EncodedRgb>(bytemuck::cast_slice(&buf));
        assert!(o.is_err());

        let buf = [14u64];
        let o = deserialize.deserialize::<EncodedRgb>(bytemuck::cast_slice(&buf));
        assert!(o.is_err());

        let buf = [31.0f32];
        let o = deserialize.deserialize::<EncodedRgb>(bytemuck::cast_slice(&buf));
        // lol, i don't like this. is there a way to make this not work? if you see this
        // and know the answer, please PR me!
        assert!(o.is_ok());
    }
}
