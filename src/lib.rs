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
//! called `EncodedColor`. If you use photoshop and use the color picker on a color (generally),
//! the number you get out is going to be in encoded sRGB, which this library handles in EncodedColor.
//! That "generally" might have worried you; unless you know you did something odd, however, it shouldn't.
//! If you're authoring texture in Photoshop or in Aseprite, you'll be working in sRGB (unless you make
//! it so you aren't, but don't do that).
//!
//! Encoded sRGB is just the bee's knees, except that it's basically useless to *do* things in.
//! When you want to *blend* colors (add them, multiply them, basically do anything to them),
//! you need to convert those colors into "linear" space. In this library, we call this `LinearColor`.
//! Whereas `EncodedColor` is just 4 u8s, `LinearColor` is 4 f32s, each of which has been transferred
//! from "encoded" space to "linear" space. The more complete terms would be that they have been
//! transferred from "encoded sRGB" to "linear sRGB", but don't think about it too much -- basically,
//! now they're in a place where they can be mixed with each other.
//!
//! # When does this happen Magically?
//!
//! Most of the time, in your library or application, your colors will be in `EncodedColor`
//! and you won't think much about it. If you use a tool like egui or imgui-rs, you'll set colors
//! from those color picker applets directly into your `EncodedColor` and call it a day.
//! In fact, if you're working in something like Opengl or Vulkan, and you're passing in Colors
//! in a Vertex Attribute, you may *still* use `EncodedColor` in that circumstance, so long as
//! you make sure to make that attribute normalized correctly (in [vulkan](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkFormat.html),
//! and in [opengl](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glVertexAttribPointer.xhtml)).
//!
//! And of course, I've said a few times now that Textures are in EncodedColor, yet, of course,
//! when you access them in a Shader, you can tint them with uniforms easily and correctly,
//! so they must also be in linear at that stage, right?
//!
//! The answer is yes! The GPU, when it samples a texture, will convert it into LinearColor *for you.*
//! It will also, if you've set up your vertex attributes like above, do the same for those.
//!
//! Even more confusingly, after your fragment shader is done working in linear colors, it will (generally)
//! be converted *back* into EncodedColor for final output. This is why if you use a color picker on your screen,
//! you'll still be getting EncodedColor colors out! If your monitor itself is in sRgb (and many are), then you'll
//! even be displaying those colors in EncodedColor.
//!
//! # When do I need to transfer EncodedColor to LinearColor myself?
//!
//! In two circumstances, for most programmers -- when you're blending colors yourself on the CPU, or when
//! you're sending a color to a uniform to be blended with another LinearColor color (like a sampled texture) on the GPU.
//!
//! You might think to yourself that you commonly sent colors before you read this in "what you're calling 'EncodedColor'" and
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

#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
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
/// color space into a different color, [LinearColor], with [to_linear](Self::to_linear)
/// before we do such operations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[repr(C)]
pub struct EncodedColor {
    /// The red component of the color.
    pub r: u8,

    /// The green component of the color.
    pub g: u8,

    /// The blue component of the color.
    pub b: u8,

    /// The alpha component of the color, normally the opacity in blending operations.
    pub a: u8,
}

impl EncodedColor {
    /// A basic white (255, 255, 255, 255) with full opacity.
    pub const WHITE: EncodedColor = EncodedColor::new(255, 255, 255, 255);

    /// A basic black (0, 0, 0, 255) with full opacity.
    pub const BLACK: EncodedColor = EncodedColor::new(0, 0, 0, 255);

    /// A black (0, 0, 0, 0) with zero opacity.
    pub const CLEAR: EncodedColor = EncodedColor::new(0, 0, 0, 0);

    /// God's color (255, 0, 255, 255). The color of choice for graphics testing.
    pub const FUCHSIA: EncodedColor = EncodedColor::new(255, 0, 255, 255);

    /// Creates a new encoded 32bit color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Transforms this color into the Linear color space.
    #[inline]
    pub fn to_linear(self) -> LinearColor {
        LinearColor {
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
    #[inline]
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
    #[inline]
    pub fn from_encoded_f32s(input: [f32; 4]) -> Self {
        Self::new(
            (input[0] * 255.0) as u8,
            (input[1] * 255.0) as u8,
            (input[2] * 255.0) as u8,
            (input[3] * 255.0) as u8,
        )
    }

    /// Converts a packed u32 to an encoded rgba struct.
    ///
    /// Note, your colors must be in order of `red, green, blue, alpha`. For `bgra` support,
    /// use `from_bgra_u32`.
    ///
    /// This function might also has issues on non-little endian platforms, but look, you're not
    /// on one of those.
    #[inline]
    pub const fn from_rgba_u32(input: u32) -> Self {
        let bytes = input.to_ne_bytes();

        Self {
            r: bytes[3],
            g: bytes[2],
            b: bytes[1],
            a: bytes[0],
        }
    }

    /// Converts the encoded rgba struct to a packed u32 in `rgba` encoding.
    ///
    /// This will output your colors in order of `red, green, blue, alpha`. For `bgra` support,
    /// use `to_bgra_u32`.
    ///
    /// This function might also have issues on non-little endian platforms, but look, you're not
    /// on one of those.
    #[inline]
    pub const fn to_rgba_u32(self) -> u32 {
        let mut bytes = [0, 0, 0, 0];

        bytes[3] = self.r;
        bytes[2] = self.g;
        bytes[1] = self.b;
        bytes[0] = self.a;

        u32::from_ne_bytes(bytes)
    }

    /// Converts a packed u32 to an encoded rgba struct. On little endian platforms, this is a no-op.
    ///
    /// Note, your colors must be in order of `blue`, `green`, `red`, `alpha`.
    ///
    /// This function might also has issues on non-little endian platforms, but look, you're not
    /// on one of those probably.
    #[inline]
    pub const fn from_bgra_u32(input: u32) -> Self {
        let bytes = input.to_ne_bytes();

        Self {
            r: bytes[1],
            g: bytes[2],
            b: bytes[3],
            a: bytes[0],
        }
    }

    /// Converts the encoded rgba struct to a packed u32 in `bgra` encoding.
    ///
    /// This will output your colors in order of `red, green, blue, alpha`. For `bgra` support,
    /// use `to_bgra_u32`.
    ///
    /// This function might also have issues on non-little endian platforms, but look, you're not
    /// on one of those.
    #[inline]
    pub const fn to_bgra_u32(self) -> u32 {
        let mut bytes = [0, 0, 0, 0];

        bytes[1] = self.r;
        bytes[2] = self.g;
        bytes[3] = self.b;
        bytes[0] = self.a;

        u32::from_ne_bytes(bytes)
    }

    /// Recasts four u8s into `EncodedColor`
    pub const fn from_bits_u32(value: u32) -> Self {
        unsafe { core::mem::transmute(value) }
    }

    /// Recasts four u8s into `EncodedColor`
    pub const fn from_bits(value: [u8; 4]) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

impl From<(u8, u8, u8, u8)> for EncodedColor {
    fn from(o: (u8, u8, u8, u8)) -> Self {
        Self {
            r: o.0,
            g: o.1,
            b: o.2,
            a: o.3,
        }
    }
}

impl From<EncodedColor> for (u8, u8, u8, u8) {
    fn from(o: EncodedColor) -> Self {
        (o.r, o.g, o.b, o.a)
    }
}

impl fmt::Debug for EncodedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EncodedColor")
            .field(&self.r)
            .field(&self.g)
            .field(&self.b)
            .field(&self.a)
            .finish()
    }
}

impl fmt::Display for EncodedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "r: {}, g: {}, b: {}, a: {}, {:x}{:x}{:x}{:x}",
            self.r, self.g, self.b, self.a, self.r, self.g, self.b, self.a
        )
    }
}

// we use rgba encoding, for simplicity...
impl fmt::LowerHex for EncodedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_rgba_u32();

        fmt::LowerHex::fmt(&val, f)
    }
}

// we use rgba encoding, for simplicity...
impl fmt::UpperHex for EncodedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_rgba_u32();

        fmt::UpperHex::fmt(&val, f)
    }
}

/// This is a Color in the `linear` space. This represents
/// "linear sRGB". You should use this color space when blending colors on the CPU
/// or when sending uniforms to a linear card.
///
/// Colors on disc are [EncodedColor], but to blend them correctly, you need to move them
/// into the `linear` color space with [to_linear](EncodedColor::to_linear).
///
/// You *can* directly create this struct, but you probably don't want to. You'd need already
/// linear sRGB to correctly make this struct -- that's possible to have, but generally, textures,
/// color pickers (like photoshop), and outputted surface (like if you use a Color Picker on a game)
/// will all be in the encoded RGB space. Exceptions abound though, so it is possible to directly
/// create this color.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct LinearColor {
    /// The red component of the color.
    pub r: f32,

    /// The green component of the color.
    pub g: f32,

    /// The blue component of the color.
    pub b: f32,

    /// The alpha component of the color, normally the opacity in blending operations.
    pub a: f32,
}

impl LinearColor {
    /// **You probably don't want to use this function.**
    /// This creates a color in the LinearColor space directly. For this function to be valid,
    /// the colors given to this function **must be in the linear space already.**
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Transforms this color into the Encoded color space. Use this space to serialize
    /// colors.
    #[inline]
    pub fn to_encoded_space(self) -> EncodedColor {
        EncodedColor {
            r: linear_to_encoded(self.r),
            g: linear_to_encoded(self.g),
            b: linear_to_encoded(self.b),
            a: (self.a * 255.0) as u8,
        }
    }

    /// Creates an array representation of the color. This is useful for sending the color
    /// to a uniform, but is the same memory representation as `Self`. [LinearColor] also implements
    /// Into, but this function is often more convenient.
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        self.into()
    }

    /// Encodes the 4 floats as 16 u8s. This is useful for sending the color
    /// to a uniform, but is the same memory representation as `Self` -- ie,
    /// the bits have just been reinterpreted as 16 u8s, but they're still secret floats.
    #[inline]
    pub fn to_bits(self) -> [u8; 16] {
        unsafe { core::mem::transmute(self.to_array()) }
    }

    /// Recasts four u8s into floats. Note: these floats could be subnormal if these u8s
    /// were produced incorrectly.
    pub fn from_bits(value: [u8; 16]) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

impl From<LinearColor> for [f32; 4] {
    fn from(o: LinearColor) -> Self {
        [o.r, o.g, o.b, o.a]
    }
}

impl From<[f32; 4]> for LinearColor {
    fn from(o: [f32; 4]) -> Self {
        Self::new(o[0], o[1], o[2], o[3])
    }
}

impl From<LinearColor> for (f32, f32, f32, f32) {
    fn from(o: LinearColor) -> Self {
        (o.r, o.g, o.b, o.a)
    }
}

impl From<(f32, f32, f32, f32)> for LinearColor {
    fn from(o: (f32, f32, f32, f32)) -> Self {
        Self::new(o.0, o.1, o.2, o.3)
    }
}

impl fmt::Debug for LinearColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LinearColor")
            .field(&self.r)
            .field(&self.g)
            .field(&self.b)
            .field(&self.a)
            .finish()
    }
}

impl fmt::Display for LinearColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "r: {}, g: {}, b: {}, a: {}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl From<LinearColor> for EncodedColor {
    fn from(o: LinearColor) -> Self {
        o.to_encoded_space()
    }
}

impl From<EncodedColor> for LinearColor {
    fn from(o: EncodedColor) -> Self {
        o.to_linear()
    }
}

/// This function takes an encoded u8 and outputs a linear space (linear) sRgb f32.
///
/// This is based on <https://bottosson.github.io/posts/colorwrong/> and similar
/// transfer functions.
///
/// We do this with a LUT rather than actually calculate it, since it's just faster to do
/// an array lookup than do the math.
///
/// However, in the interest of simplicity, I have left the math which does the conversion commented
/// within this function call, if you're like to see the math.
pub const fn encoded_to_linear(c: u8) -> f32 {
    ENCODED_TO_LINEAR_LUT[c as usize]

    /*
    If you want to see the encoded to linear function written out (ie, how I made this LUT),
    it looks like this here:

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

    Thank you very much to @thomcc (@zurr on discord) for helping me with this!
    */
}

/// This is the LUT that we use. You shouldn't really ever need to use directly, but `encoded_to_linear`
/// is just a wrapper to index into this LUT.
/// 
/// I have chosen to inline write this, rather than use a build script, because it's a bit simpler.
#[rustfmt::skip]
pub const ENCODED_TO_LINEAR_LUT: [f32; 256] = [
    0.0, 0.000303527, 0.000607054, 0.000910581, 0.001214108, 0.001517635, 0.001821162, 0.0021246888,
    0.002428216, 0.0027317428, 0.00303527, 0.0033465358, 0.0036765074, 0.004024717, 0.004391442,
    0.0047769533, 0.0051815165, 0.0056053917, 0.006048833, 0.0065120906, 0.00699541, 0.007499032,
    0.008023193, 0.008568126, 0.009134059, 0.009721218, 0.010329823, 0.010960094, 0.011612245,
    0.012286488, 0.0129830325, 0.013702083, 0.014443844, 0.015208514, 0.015996294, 0.016807375,
    0.017641954, 0.01850022, 0.019382361, 0.020288562, 0.02121901, 0.022173885, 0.023153367,
    0.024157632, 0.02518686, 0.026241222, 0.027320892, 0.02842604, 0.029556835, 0.030713445,
    0.031896032, 0.033104766, 0.034339808, 0.035601314, 0.03688945, 0.038204372, 0.039546236,
    0.0409152, 0.04231141, 0.04373503, 0.045186203, 0.046665087, 0.048171826, 0.049706567,
    0.051269457, 0.052860647, 0.054480277, 0.05612849, 0.05780543, 0.059511237, 0.061246052,
    0.063010015, 0.064803265, 0.06662594, 0.06847817, 0.070360094, 0.07227185, 0.07421357,
    0.07618538, 0.07818742, 0.08021982, 0.08228271, 0.08437621, 0.08650046, 0.08865558, 0.09084171,
    0.093058966, 0.09530747, 0.09758735, 0.099898726, 0.10224173, 0.104616486, 0.107023105, 0.10946171,
    0.11193243, 0.114435375, 0.116970666, 0.11953843, 0.122138776, 0.12477182, 0.12743768, 0.13013647,
    0.13286832, 0.13563333, 0.13843161, 0.14126329, 0.14412847, 0.14702727, 0.14995979, 0.15292615,
    0.15592647, 0.15896083, 0.16202937, 0.1651322, 0.1682694, 0.17144111, 0.1746474, 0.17788842, 0.18116425,
    0.18447499, 0.18782078, 0.19120169, 0.19461784, 0.19806932, 0.20155625, 0.20507874, 0.20863687,
    0.21223076, 0.2158605, 0.2195262, 0.22322796, 0.22696587, 0.23074006, 0.23455058, 0.23839757, 0.24228112,
    0.24620132, 0.25015828, 0.2541521, 0.25818285, 0.26225066, 0.2663556, 0.2704978, 0.2746773, 0.27889428,
    0.28314874, 0.28744084, 0.29177064, 0.29613826, 0.30054379, 0.3049873, 0.30946892, 0.31398872, 0.31854677,
    0.3231432, 0.3277781, 0.33245152, 0.33716363, 0.34191442, 0.34670407, 0.3515326, 0.35640013, 0.3613068,
    0.3662526, 0.3712377, 0.37626213, 0.38132602, 0.38642943, 0.39157248, 0.39675522, 0.40197778, 0.4072402,
    0.4125426, 0.41788507, 0.42326766, 0.4286905, 0.43415365, 0.43965718, 0.4452012, 0.4507858, 0.45641103,
    0.462077, 0.4677838, 0.47353148, 0.47932017, 0.48514995, 0.49102086, 0.49693298, 0.5028865, 0.50888133,
    0.5149177, 0.52099556, 0.5271151, 0.5332764, 0.5394795, 0.54572445, 0.55201143, 0.5583404, 0.5647115,
    0.57112485, 0.57758045, 0.58407843, 0.59061885, 0.59720176, 0.60382736, 0.61049557, 0.6172066, 0.6239604,
    0.63075715, 0.63759685, 0.6444797, 0.65140563, 0.65837485, 0.6653873, 0.67244315, 0.6795425, 0.6866853,
    0.69387174, 0.7011019, 0.70837575, 0.7156935, 0.7230551, 0.73046076, 0.7379104, 0.7454042, 0.7529422,
    0.7605245, 0.76815116, 0.7758222, 0.7835378, 0.7912979, 0.7991027, 0.80695224, 0.8148466, 0.82278574,
    0.8307699, 0.838799, 0.8468732, 0.8549926, 0.8631572, 0.8713671, 0.8796224, 0.8879231, 0.8962694, 0.9046612,
    0.91309863, 0.92158186, 0.9301109, 0.9386857, 0.9473065, 0.9559733, 0.9646863, 0.9734453, 0.9822506,
    0.9911021, 1.0,
];

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

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for EncodedColor {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for EncodedColor {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for LinearColor {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for LinearColor {}

#[cfg(feature = "serde")]
const ENCODED_NAME: &str = "Encoded Rgb";

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

#[cfg(feature = "rand")]
impl rand::distributions::Distribution<EncodedColor> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> EncodedColor {
        EncodedColor {
            r: rng.gen(),
            g: rng.gen(),
            b: rng.gen(),
            a: rng.gen(),
        }
    }
}

#[cfg(feature = "rand")]
impl rand::distributions::Distribution<LinearColor> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> LinearColor {
        LinearColor {
            r: rng.gen(),
            g: rng.gen(),
            b: rng.gen(),
            a: rng.gen(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static_assertions::assert_eq_align!(EncodedColor, u8);
    static_assertions::assert_eq_size!(EncodedColor, [u8; 4]);

    #[test]
    fn from_u32s() {
        let cornwall_blue_in_rgba: u32 = 0x6b9ebeff;
        let cornwall_blue_in_bgra: u32 = 0xbe9e6bff;
        let cornwall_encoded = EncodedColor {
            r: 107,
            g: 158,
            b: 190,
            a: 255,
        };
        let encoded_rgba = EncodedColor::from_rgba_u32(cornwall_blue_in_rgba);
        assert_eq!(encoded_rgba, cornwall_encoded);
        assert_eq!(encoded_rgba.to_rgba_u32(), cornwall_blue_in_rgba);

        let encoded_bgra = EncodedColor::from_bgra_u32(cornwall_blue_in_bgra);
        assert_eq!(encoded_bgra, cornwall_encoded);
        assert_eq!(encoded_bgra.to_bgra_u32(), cornwall_blue_in_bgra);

        #[cfg(feature = "std")]
        {
            // and finally, check the hex...
            let rgba_as_hex = std::format!("{:x}", encoded_rgba);
            assert_eq!(rgba_as_hex, "6b9ebeff");

            let rgba_as_hex = std::format!("{:#X}", encoded_rgba);
            assert_eq!(rgba_as_hex, "0x6B9EBEFF");
        }
    }

    #[test]
    fn encoding_decoding() {
        fn encode(input: u8, output: f32) {
            let o = encoded_to_linear(input);
            assert!((o - output).abs() < f32::EPSILON);
        }

        fn decode(input: f32, output: u8) {
            let o = linear_to_encoded(input);
            assert_eq!(o, output);
        }

        encode(66, 0.05448028);
        encode(0, 0.0);
        encode(255, 1.0);
        encode(240, 0.8713671);
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
    #[allow(clippy::float_cmp)]
    #[cfg(feature = "std")]
    fn test_lut() {
        // does computation with 64 bits of precision since we can spare it for
        // the LUT
        fn srgb_to_linear_high_precision(component: u8) -> f32 {
            let c = component as f64 / 255.0;
            (if c > 0.04045 {
                ((c + 0.055) / 1.055).powf(2.4)
            } else {
                c / 12.92
            }) as f32
        }

        let expect = (0..=255u8)
            .map(srgb_to_linear_high_precision)
            .collect::<std::vec::Vec<_>>();
        assert_eq!(expect, ENCODED_TO_LINEAR_LUT);
        for c in 0..=255u8 {
            assert_eq!(encoded_to_linear(c), expect[c as usize]);
        }
    }

    #[test]
    #[cfg(feature = "serde")]
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
