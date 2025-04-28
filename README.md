# Bevy DoG
## Difference of Gaussians
Difference of Gaussians is a versatile tool in creative coding. It can be used for edge detection, thresholding and some other effects. 
Please take a look at [this paper](https://users.cs.northwestern.edu/~sco590/winnemoeller-cag2012.pdf) for how it works in detail. [This Video by Acerola](https://www.youtube.com/watch?v=5EuYKEvugLU) combined with their [provided code](https://github.com/GarrettGunnell/Post-Processing/tree/main/Assets/Edge%20Detection) inspired me to try and bring this to bevy.

## Usage
Here's a minimal usage example:
```toml
# Cargo.toml
[dependencies]
bevy = "0.16.0"
bevy_dog = { git = "https://github.com/arne-berner/bevy_dog", branch = "main" }
```

```rust
use bevy::prelude::*;
use bevy_dog::{
    plugin::DoGPlugin,
    settings::{DoGSettings, PassesSettings},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DoGPlugin))
        .run();
}

fn setup(
    mut commands: Commands,
) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(5.0, 3.0, 0.0)).looking_at(Vec3::default(), Vec3::Y),
        DoGSettings::default(),
        PassesSettings::default(),
    ));
}

```

## Examples
To run an example, use the following command (you may replace `ui` with a name of another example):

```bash
cargo run --example ui
```

### Simple Scene
The simple scene creates a sphere, a rotating cube and a plane. It also adds some very basic ui to change the DoGSettings.

## Bevy support table
| bevy | bevy_dog  |
|------|-----------|
| 0.16 | 0.1       |
