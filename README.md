<div align="center">
  <h2 align="center">Elden Ring mod to style phantom colors</h2>
  <p align="center">
    <img width="790" height="444" alt="image" src="https://github.com/user-attachments/assets/6d3c9775-0d96-47b5-8869-650fc197de7e" />
  </p>
  Implements a live-updating config editor that makes adjustments to the game's PhantomParam system.
</div>

## Made with
- [Fromsoftware-rs](https://github.com/vswarte/fromsoftware-rs)
- [Notify](https://github.com/notify-rs/notify)
- [tracing](https://github.com/tokio-rs/tracing) & [tracing-subscriber](https://github.com/tokio-rs/tracing/tree/main/tracing-subscriber)
- [serde](https://github.com/serde-rs/serde) & [toml](https://github.com/toml-rs/toml)
- [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam)
- [Rune](https://github.com/rune-rs)
- [Windows](https://github.com/microsoft/windows-rs)

## Requirements
- [Mod Engine 3](https://github.com/garyttierney/me3/)
- Windows probably, due to how i get the profile PathBuf.

## Installing
1. Install [Mod Engine 3](https://github.com/garyttierney/me3/releases/latest)
2. Install the Dll from the [latest release](https://github.com/lndura/technicolor-tarnished/releases/latest)
4. Add the Dll to a [Mod Engine 3 profile](https://me3.help/en/latest/configuration-reference/#what-is-a-modprofile-configuration) as a [native](https://me3.help/en/latest/configuration-reference/#native-object).
5. Add a `phantom_color_profile.toml` relative to where you've moved the Dll.
6. Adjust the profile and add Rune scripts as you like.

### Profile configurations
```toml
# This will throttle ChrSet iterations and live-reload Polling by the given amount of milliseconds.
interval = 5000

# Any of these entries apply a rune script to all Param Id's they reference.
[[script]]
param_id_list = [
    60, # Invaders
]
script_path = "my-script.rn"

# Any of these entries are applied to all Param Id's they reference.
[[param]]
param_id_list = [
    902, # Bloodstain Ghost
]
edge_color_a = 0.1
front_color_a = 0.1
diff_mul_color_a = 0.1
spec_mul_color_a = 0.1
light_color_a = 0.1

edge_color_r = 10
edge_color_g = 0
edge_color_b = 200

front_color_r = 10
front_color_g = 10
front_color_b = 10

diff_mul_color_r = 10
diff_mul_color_g = 10
diff_mul_color_b = 10

spec_mul_color_r = 10
spec_mul_color_g = 10
spec_mul_color_b = 10

light_color_r = 10
light_color_g = 10
light_color_b = 10

alpha = 1.0
blend_rate = 1.0
blend_type = 0
is_edge_subtract = 0
is_front_subtract = 0
is_no2_pass = 0
edge_power = 1.0
glow_scale = 1.0

# Any of these entries are applied to all Character Id's they reference.
[[chr_id]]
param_id = 61 # Golden summon
chr_id_list = [
    8000 # Torrent
]
edge_color_a = 0.1
front_color_a = 0.1
diff_mul_color_a = 0.1
spec_mul_color_a = 0.1
light_color_a = 0.1

edge_color_r = 255
edge_color_g = 0
edge_color_b = 0

front_color_r = 10
front_color_g = 10
front_color_b = 10

diff_mul_color_r = 10
diff_mul_color_g = 10
diff_mul_color_b = 10

spec_mul_color_r = 10
spec_mul_color_g = 10
spec_mul_color_b = 10

light_color_r = 10
light_color_g = 10
light_color_b = 10

alpha = 1.0
blend_rate = 1.0
blend_type = 0
is_edge_subtract = 0
is_front_subtract = 0
is_no2_pass = 0
edge_power = 1.0
glow_scale = 1.0

# This entry is applied to the player every frame
[player]
param_id = 60 # Invader
edge_color_a = 0.1
front_color_a = 0.1
diff_mul_color_a = 0.1
spec_mul_color_a = 0.1
light_color_a = 0.1

edge_color_r = 255
edge_color_g = 0
edge_color_b = 0

front_color_r = 10
front_color_g = 10
front_color_b = 10

diff_mul_color_r = 10
diff_mul_color_g = 10
diff_mul_color_b = 10

spec_mul_color_r = 10
spec_mul_color_g = 10
spec_mul_color_b = 10

light_color_r = 10
light_color_g = 10
light_color_b = 10

alpha = 1.0
blend_rate = 1.0
blend_type = 0
is_edge_subtract = 0
is_front_subtract = 0
is_no2_pass = 0
edge_power = 1.0
glow_scale = 1.0

# This entry is applied to all summons every frame
[summon]
param_id = 211
edge_color_a = 0.1
front_color_a = 0.1
diff_mul_color_a = 0.1
spec_mul_color_a = 0.1
light_color_a = 0.1

edge_color_r = 10
edge_color_g = 150
edge_color_b = 150

front_color_r = 10
front_color_g = 10
front_color_b = 10

diff_mul_color_r = 10
diff_mul_color_g = 10
diff_mul_color_b = 10

spec_mul_color_r = 10
spec_mul_color_g = 10
spec_mul_color_b = 10

light_color_r = 10
light_color_g = 10
light_color_b = 10

alpha = 1.0
blend_rate = 1.0
blend_type = 0
is_edge_subtract = 0
is_front_subtract = 0
is_no2_pass = 0
edge_power = 1.0
glow_scale = 1.0
```

### Example Rune script
Below is what `my-script.rn` references:
```rust
pub fn main(param) {
    let min = 0;
    let max = 255;
    let step = 1;

    if param.edge_color_r == max && param.edge_color_b == min && param.edge_color_g < max {
        param.edge_color_g = param.edge_color_g + step;
    }

    if param.edge_color_g == max && param.edge_color_b == min && param.edge_color_r > min {
        param.edge_color_r = param.edge_color_r - step;
    }

    if param.edge_color_g == max && param.edge_color_r == min && param.edge_color_b < max {
        param.edge_color_b = param.edge_color_b + step;
    }

    if param.edge_color_b == max && param.edge_color_r == min && param.edge_color_g > min {
        param.edge_color_g = param.edge_color_g - step;
    }

    if param.edge_color_b == max && param.edge_color_g == min && param.edge_color_r < max {
        param.edge_color_r = param.edge_color_r + step;
    }

    if param.edge_color_r == max && param.edge_color_g == min && param.edge_color_b > min {
        param.edge_color_b = param.edge_color_b - step;
    }

    param
}
```
For detailed documentation and guides see the [Rune examples](https://github.com/rune-rs/rune/tree/main/examples)
