# randl
A param randomization framework for Smash Ultimate

<img src="https://user-images.githubusercontent.com/8260240/126084219-5c15e13a-21b0-4c5a-89c4-0f0b9417fe12.png" width="200px" height="200px">

## Usage

anyways, the configs go in

```
sd:/atmosphere/contents/01006A800016E000/romfs/randl
```

it loads ever .kdl file in that folder
currently only 2 types of prc paths are supported:

1. absolute paths (example: fighter/captain/param/vl.prc)
2. absolute templated paths (example: fighter/{fighter_names}/param/vl.prc)

the way the templating works:

```
set "fighter_names" {
    value "captain"
    value "bayonetta"
}

file "fighter/{fighter_names}/param/vl.prc" {
    // ...
}
```

where `{fighter_names}` gets replaced by every value in the set fighter_names. meaning the above is the same as:

```
file "fighter/bayonetta/param/vl.prc" {
    // ...
}

file "fighter/captain/param/vl.prc" {
    // ...
}
```
