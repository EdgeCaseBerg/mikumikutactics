## Install


On linux:
```
sudo apt install libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev libxss-dev libxtst-dev libxkbcommon-dev libharfbuzz-dev
```
and if you'll be targeting wasm:
```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

```


On other platforms figure it out yourself here https://github.com/libsdl-org/SDL/blob/main/INSTALL.md

## Build

For native:

```
cargo build
```

For web via wasm:

```
cargo build --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/mikumikutactics.wasm --out-dir web/pkg --target web
```

If you want to build both then just run `make` and it will do the trick and copy all the files you need into the right place.
Then just serve the web folder out via a python or php server as you please.

## Side notes about the "game"

This is an expansion on a previous project whose full dev blog can be [found here](https://peetseater.space/blag/2026-05-25-miku-miku-tower-defense)
and the wasm specific work is covered [here](https://peetseater.space/blag/2026-06-19-wasm). The work to create a tactics game from the shared core
created for the turret defense game is something being slowly worked towards by the prototyping in this repo shown in [this blog post]](https://peetseater.space/blag/2026-10-29-prototyping-for-mmt.html) if you're interested. 

There is no audio controls in the game, so turn it down and then up as needed if you swap things around to trigger the VN scenes that have some audio.

## Credits

- The font used in the game is BoldPixels by YukiPixels and you can find the links to their work in the license file in the assets folder.

- Hatsune Miku the character design herself is creative commons as noted here https://piapro.net/intl/en_for_creators.html though most people know her for her synthesizer which is owned by Crypton Future Media and Sony.

## Enhancments / exercise for the bored

The entire test_screen is one giant exercise in prototyping to see what sort of works and what doesn't, so there's plenty to be played with by the bored.
Check out the prototyping blogpost's final section for some of the thoughts and feelings there. 
