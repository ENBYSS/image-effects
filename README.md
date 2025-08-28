# Image Effects

This project is essentially a library for applying effects onto images. It makes heavy use of the `image` crate since it's the main image processing library for rust, and `palette` for helping with colour conversions and more - although the project started off with a custom made colour library.

Currently there's two major classes of effect, **dithering** which works on images and supports both **ordered** and **error propagation** methods, and **filters** such as brightness, contrast, saturation, etc. which work on individual pixels _(and therefore also on images)_.

The following shows which effects are supported on what types.

|         | dither | filter |
| ------- | ------ | ------ |
| `pixel` | no     | yes    |
| `image` | yes    | yes    |
| `gif`   | yes    | yes    |

- `pixel` = `[u8; 3]` / `[u8; 4]`
- `image` = `Vec<Vec<{pixel}>>` / (image) `DynamicImage` / (image) `ImageBuffer<Rgb(a)<u8>, Vec<u8>>`
- `gif` = (image) `Frame`

_(image): from the `image` crate._

## Dithering

### Methodology

The **1-bit** dithering is separated purely for convenience, since needing to specify a black and white palette every time can be a bit annoying. Internally it just calls the normal error propagation function with a **one-bit** palette.

**Bayer** and **Basic** are separated from the rest in implementation due to how differently they work. **Bayer**, also known as _ordered dithering_, doesn't propagate any errors and just manipulates each pixel on the spot using a matrix. **Basic**, being the _naive_ implementation, simply propagates the error to the right.

All other algorithms are variations on **error propagation** by using different parameters. As such the implementation works by creating a general error propagation function which takes:

- a `list` of propagation offsets - aka where to send the error and _by how much_. For example if a matrix offers **30** divisions, `(1, 0, 5)` would point to the _next_ pixel `(x+1, y)` and send over **5/30ths** of the error.
- the _amount of division_. Most algorithms split the error totally - but some like _Atkison_ only propagate _some_.

After this, each of these algorithms were effectively generated using a macro.

There's two kinds of dithering: **Error propagation**, and **Ordered/Bayer**. The only similarity here is that they both will go pixel by pixel, turning them into the closest match in the palette. Measuring which colour is the closest match is done in a different module - `colour` - more details [here](#colours).

#### Error Propagation

For these algorithms, I want to thank [Tanner Helland](https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html) and [Efron Licht](https://docs.rs/dither/latest/dither/) for their resources. Tanner made a write up on error propagation, including multiple methods that helped to understand the process, and Efron wrote the `dither` crate which served as an inspiration for this.

So, for _error propagation_ algorithms, after they find the closest match they will then calculate the **error** - which is essentially the `RGB` difference between the chosen colour and actual colour. This error is then
propagated to nearby pixels according to what I call the **propagation matrix** and **portion amount**.

The **propagation matrix** is more like a list of coordinates, in addition to how much of the error to propagate - in the form of `(dx, dy, portion)`. For example, `(1, 0, 5)` will send $\frac{5}{N}$ of the error to the next pixel on the right, where $N$ is the **portion amount**. Keep in mind that the error _does not need_ to be distributed exactly - for example **Atkinson** uses 8 portions, but only propagates 6 of them. You can technically also _over-propagate_, though then you're just adding extra error to the pixels.

#### Ordered / Bayer

This one works very differently and starts to delve a lot more into the math. For example, here's part of the entire algorithm:

$$
c' = \textrm{nearest\_palette\_color}(c + r \times (M(x \textrm{ mod } y, y \textrm{ mod } n) - 1/2))
$$

Here, $c'$ is the new colour, $M$ is the _threshold map_, and $r$ is the amount of spread in color space. I use $r = \frac{255}{3}$ - though in the code it's $\frac{1}{3}$ since rgb values between `0.0` and `1.0` are used. If I'm honest, I don't fully remember where I got this value of $r$ from, but it seems to work quite well.

As for the _threshold map_, it can be pre-calculated - as the only variable there is the matrix size, which usually comes in powers of two. For more on this, check out [the wikipedia page](https://en.wikipedia.org/wiki/Ordered_dithering) on ordered dithering. They can be pre-calculated, but this library supports _any arbitrary size_.

### Algorithms

Currently supports the following dithering algorithms:

|            **Name** | _1-bit_                                         | _RGB (Web-safe)_                                    | _RGB (8-bit)_                                    |
| ------------------: | :---------------------------------------------- | :-------------------------------------------------- | :----------------------------------------------- |
|     Floyd-Steinberg | ![](./data/dither/floyd-steinberg-mono.png)     | ![](./data/dither/floyd-steinberg-web-safe.png)     | ![](./data/dither/floyd-steinberg-8-bit.png)     |
| Jarvis-Judice-Ninke | ![](./data/dither/jarvis-judice-ninke-mono.png) | ![](./data/dither/jarvis-judice-ninke-web-safe.png) | ![](./data/dither/jarvis-judice-ninke-8-bit.png) |
|              Stucki | ![](./data/dither/stucki-mono.png)              | ![](./data/dither/stucki-web-safe.png)              | ![](./data/dither/stucki-8-bit.png)              |
|            Atkinson | ![](./data/dither/atkinson-mono.png)            | ![](./data/dither/atkinson-web-safe.png)            | ![](./data/dither/atkinson-8-bit.png)            |
|              Burkes | ![](./data/dither/burkes-mono.png)              | ![](./data/dither/burkes-web-safe.png)              | ![](./data/dither/burkes-8-bit.png)              |
|              Sierra | ![](./data/dither/sierra-mono.png)              | ![](./data/dither/sierra-web-safe.png)              | ![](./data/dither/sierra-8-bit.png)              |
|        SierraTwoRow | ![](./data/dither/sierra-two-row-mono.png)      | ![](./data/dither/sierra-two-row-web-safe.png)      | ![](./data/dither/sierra-two-row-8-bit.png)      |
|          SierraLite | ![](./data/dither/sierra-lite-mono.png)         | ![](./data/dither/sierra-lite-web-safe.png)         | ![](./data/dither/sierra-lite-8-bit.png)         |
|           Bayer 2x2 | ![](./data/dither/bayer-2x2-mono.png)           | ![](./data/dither/bayer-2x2-web-safe.png)           | ![](./data/dither/bayer-2x2-8-bit.png)           |
|           Bayer 4x4 | ![](./data/dither/bayer-4x4-mono.png)           | ![](./data/dither/bayer-4x4-web-safe.png)           | ![](./data/dither/bayer-4x4-8-bit.png)           |
|           Bayer 8x8 | ![](./data/dither/bayer-8x8-mono.png)           | ![](./data/dither/bayer-8x8-web-safe.png)           | ![](./data/dither/bayer-8x8-8-bit.png)           |
|         Bayer 16x16 | ![](./data/dither/bayer-16x16-mono.png)         | ![](./data/dither/bayer-16x16-web-safe.png)         | ![](./data/dither/bayer-16x16-8-bit.png)         |

In addition, there's multiple more _esoteric_ algorithms that depend on the **Ordered** method, as used by **Bayer**. Since this method just needs a matrix of values and an image, the matrix can technically contain _anything_. This is how the following algorithms work.

- Bouncing Bowtie
- Broken Spiral
- Checkered Diamonds
- Curve Path
- Diagonal Tiles
- Diagonals N
- Diamonds
- "Hardcoded"
- Marble Tile
- Modulo Snake
- Scanline
- Shiny Bowtie
- Starburst
- Zigzag

## Filters

### Methodology

For colour, certain filters such as _brightness, saturation, hue rotation_, are done by first mapping each RGB pixel to HSL or LCH.
Originally, HSL was used due to the ease of computation - however LCH is significantly more accurate in representing each of its
components.

However, `RGB -> LCH` requires more computation than `RGB -> HSL`. Currently the code requires you change it in order to use the right pixel,
but it may be worth looking into allowing the user to use HSL instead for maximal speed.

### Algorithms

Currently supports the following effects:

|         **Name** | _Image_                                |
| ---------------: | -------------------------------------- |
|    brighten +0.2 | ![](./data/colour/brighten+0.2.png)    |
|    brighten -0.2 | ![](./data/colour/brighten-0.2.png)    |
|    saturate +0.2 | ![](./data/colour/saturate+0.2.png)    |
|    saturate -0.2 | ![](./data/colour/saturate-0.2.png)    |
|     contrast 0.5 | ![](./data/colour/contrast.0.5.png)    |
|     contrast 1.5 | ![](./data/colour/contrast.1.5.png)    |
| gradient mapping | ![](./data/colour/gradient-mapped.png) |
|   rotate hue 180 | ![](./data/colour/rotate-hue-180.png)  |
|     quantize hue | ![](./data/colour/quantize-hue.png)    |
|           invert | ![](./data/colour/invert.png)          |

The scale of parameters are as follows:

- **brighten**: takes `-1.0` to `1.0`. Positive numbers boost the brightness, with **1.0** setting it to maximum luminance - and vice versa.
- **saturate**: same as **brighten** - a max chroma of **128** is assumed _(matching with `palette`'s documentation)_ to facilitate the scale.
- **contrast**: takes any float.
  - `x > 1.0` increases contrast..
  - `x > 0.0 and x < 1.0` decreases contrast.
  - _negative contrast_ obeys a similar scale, where `-1.0` is the same as `1.0` - but with each colour channel being inverted.
  - _might_ be fun to try doing contrast calculations in other spaces... something for me to look into.

Anything else has a specific type (`quantize hue`, `gradient mapping`), or acts as expected (`rotate hue`)

## Colours

At one point, I started implementing all the colour spaces and conversions between them manually. Had a whole sub-library with it - but then I discovered that someone else did it before me [_way better_](https://docs.rs/palette/latest/palette/) and ended up throwing away all of my code.

Is what I would say if I wasn't a hoarder.

All of that code is now on [`colour-exercise-rs`](https://github.com/enbyss/colour-exercise-rs) - from **RGB** to **HSL** to **LAB** to **OKLCH** _and more_ - technically not useful as a library since it's better to just use `palette`, but if you're ever interested in learning about colour - and think seeing someone stumbling through it while _they_ were learning would help - feel free to look!

One remnant of that remains however. `colour/comparisons.rs` implements multiple distance functions.

### [`colour/comparisons.rs`](./src/colour/comparisons.rs)

### Methodology

For now, the colour distance function used is **weighted euclidean**, which looks like this:

$$
f(R, G, B) = \begin{cases}
    \sqrt{2\Delta R^2 + 4\Delta G^2 + 3\Delta B^2} & \overline{R} < 128, \\
    \sqrt{3\Delta R^2 + 4\Delta G^2 + 2\Delta B^2} & \textrm{otherwise},
\end{cases}
$$

This library also has _other_ distance functions - such as `cie76`, `cie94`, and `ciede2000`. The reason why **weighted euclidean** is being used instead is mostly for efficiency _and_ because it's been deemed good enough. However the code should be easily changeable to use any of the other functions, so the function used may change / turn into an option.

`ciede2000` in particular is _significantly_ more complicated, and may be slower by an order of magnitude. It can likely become more efficient, but as is it's a bit unfeasible for use.

## Improvements

- **Generic performance improvements**
  - This library isn't _slow_ per se, but it can definitely be improved.
  - Parallellism doesn't seem to improve the **Ordered** effect much, after some benchmarking.
  - *Use `perf`*
    - `perf record -g cargo test --release`
      - follow up with `perf report -G` to see results.q
- **SIMD**
  - Using **SIMD** would definitely help in speeding things up.
  - We could also _potentially_ generate a temporary shader using the effects, that way we leveage the **GPU**.
  - For example, implementing `to_opengl()` or similar to generate shader code, that can then be executed onto an image.
