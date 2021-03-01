# smol-rgb

A smol library for (s)Rgb color handling.

## Quick-start

To use this in your project, add this to your Cargo.toml:

```toml
smol-rgb = "0.1.0"
```

no-std is supported, but requires `libm` to work, like so:

```toml
smol-rgb = { version = "0.1.0", default-features = false, features = ["libm"]  }
```

We also support two other features: `serde` and `bytemuck`. `serde` support works
across a variety of backends such as yaml, json, and bincode.

## Who is this library for?

This library is designed for the programmer who:

- is working with graphics on the GPU (such as games)
- works entirely or almost entirely with sRGB (if you don't know what that means, that's probably you),
- and doesn't care about color beyond it "just working" correctly.

This library can also serve as a good starting point to learn more complex color theory.
For users who are comfortable working in color spaces, you should check out the much more
complicated library [palette](https://github.com/Ogeon/palette). It is significantly
more complicated, but also equally more capable.

This library, on the other hand, only works with sRGB, and is designed only to help the programmer
work with sRGB in a simple manner.

## It's not always RGB, but we can make it only sRGB.

Textures, color pickers (like egui or imgui's pickers) are generally in "encoded" sRGB.
In this library, that means 4 u8s, each of which describe how much `r`, `g`, `b`, and `a`
should be in an image. On a very technical level, this is a specification called
IEC 61966-2-1:1999, but you should never remember this again. In this library, this space is
called `EncodedRgb`. If you use photoshop and use the color picker on a color (generally),
the number you get out is going to be in encoded sRGB, which this library handles in EncodedRgb.
That "generally" might have worried you; unless you know you did something odd, however, it shouldn't.
If you're authoring texture in Photoshop or in Aseprite, you'll be working in sRGB (unless you make
it so you aren't, but don't do that).

Encoded sRGB is just the bee's knees, except that it's basically useless to *do* things in.
When you want to *blend* colors (add them, multiply them, basically do anything to them),
you need to convert those colors into "linear" space. In this library, we call this `LinearColor`.
Whereas `EncodedRgb` is just 4 u8s, `LinearColor` is 4 f32s, each of which has been transferred
from "encoded" space to "linear" space. The more complete terms would be that they have been
transferred from "encoded sRGB" to "linear sRGB", but don't think about it too much -- basically,
now they're in a place where they can be mixed with each other.

## When does this happen Magically?

Most of the time, in your library or application, your colors will be in `EncodedRgb`
and you won't think much about it. If you use a tool like egui or imgui-rs, you'll set colors
from those color picker applets directly into your `EncodedRgb` and call it a day.
In fact, if you're working in something like Opengl or Vulkan, and you're passing in Colors
in a Vertex Attribute, you may *still* use `EncodedRgb` in that circumstance, so long as
you make sure to make that attribute normalized correctly (in [vulkan](https://www.khronos.org/registry/vulkan/specs/1.tensions/man/html/VkFormat.html),
and in [opengl](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glVertexAttribPointer.xhtml)).

And of course, I've said a few times now that Textures are in EncodedRgb, yet, of course,
when you access them in a Shader, you can tint them with uniforms easily and correctly,
so they must also be in linear at that stage, right?

The answer is yes! The GPU, when it samples a texture, will convert it into LinearRgb *for you.*
It will also, if you've set up your vertex attributes like above, do the same for those.
Even more confusingly, after your fragment shader is done working in linear colors, it will (generally)
be converted *back* into EncodedRgb for final output. This is why if you use a color picker on your screen,
you'll still be getting EncodedRgb colors out! If your monitor itself is in sRgb (and many are), then you'll
even be displaying those colors in EncodedRgb.

## When do I need to transfer EncodedRgb to LinearRgb myself?

In two circumstances, for most programmers -- when you're blending colors yourself on the CPU, or when
you're sending a color to a uniform to be blended with another LinearRgb color (like a sampled texture) on the GPU.
You might think to yourself that you commonly sent colors before you read this in "what you're calling 'EncodedRgb'" and
it worked out just fine. That's probably true! Almost all games have some color error, because it's just so easy to do
accidentally. However, I might point out that probably you or an artist just fiddled with the encoded color until it
mixed correctly, so it looked more or less right on the GPU. Or perhaps there was some other weirdness going on!

## A quick final note on alpha

This library uses the term `Rgb` for its color space, which is really `sRgb` with an `alpha` channel.
We do this for the sake of simplicity -- alpha is almost always desired in grapics applications, and like, come on,
you can spare the byte.
If this library picks up enough traction, users might want to split it into `Rgb` and `Rgba`. Leave an issue
if that's desired.

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
