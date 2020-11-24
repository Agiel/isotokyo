# Isotokyo
Isometric tactical shooter inspired by the Source mod [Neotokyo](https://store.steampowered.com/app/244630/NEOTOKYO/), featuring Quake/Source style movement and point-and-click aiming with screen space recoil.

![screenshot](screenshot.png)

## Technology
Rendering using [wgpu-rs](https://github.com/gfx-rs/wgpu-rs) with architecture partially lifted from [vange-rs](https://github.com/kvark/vange-rs).

ECS, networking and sound TBD.

## Config
A config file will be created in `config/config.ron` when the game is launched for the first time. The settings should be mostly self-explanatory.
