# Wallflower: A wallpaper replacement for macOS and Wayland (support for other platforms in the future!)

Wallflower is a layer-based wallpaper compositor written in Rust. 

## Feature map

- [ ] Implement core renderer
- [ ] Fully define the TOML format for defining wallpapers
- [ ] Create GUI to edit wallpapers
- [ ] Add RustPython integration

## End goal

All wallpapers have three things:

A list of sources, a list of layers, and an order for those layers.

```toml
# The various sources required. They can be reused in multiple layers. Ideally, custom types can be added.
[sources]
black = {
    type = "fill"
    color = "rgb(0,0,0)"
}

# Layers define what to render, how, and where.
[layer.black]
source = "black"

# You have to define the order to render, layers can be reused.
[layer]
order = ["black"]
```

This very simple wallpaper can be shrunk to the following:

```toml
# All wallpapers have a root layer, but by default it doesn't get rendered. 
[layer.root]
# Sources can be defined inline, however this is not recommended in most cases.
type = "fill"
color = "rgb(0,0,0)

```

Ideally you can do pretty much whatever, a WHOLE bunch of generators can be provided. Ideally in the future it'll even be possible to render stuff directly to the layer.


```toml

[sources]
# Sources can be a lot of things!
blueprint = {
    type = "net/get",
    url = "apollo.quackthe.duckspin.net",
}
rainbow = "gen/rainbow"

[layer.blueprint]
contents = "blueprint"
# Blend modes and conditionals!
blend = {
    theme.light = "xor",
    theme.dark = "and"
}


[layer]
# Ordering is top to bottom
order = ["blueprint", "source.rainbow"]
```