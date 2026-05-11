# Yggdrasil Vanity

Vanity address generator for the Yggdrasil network.

## Binary builds

You can download binary builds for Windows and Linux in the [releases](https://github.com/averyanalex/yggdrasil-vanity/releases).

## Basic usage (just search high addresses)

```shell
# build and run from source
cargo run --release
# or download and run
./yggdrasil-vanity
```

## Regex matching

Pass -r "regex" argument (you can do it multiple times of you want search for multiple patterns) to search only for adresses matching given regex.

Example:

```shell
./yggdrasil-vanity -r "" -r "^([0-9a-f]*:){2}:" -r "^([0-9a-f]*:){2}[0-9a-f]{0,2}:0:" -r "^([0-9a-f]*:){3}0:" -r "1234:5678"
# or use cargo run --release -- -r "" ... if you want to build from source
```

## Benchmarks

- AMD Radeon RX 6800 XT: 25 MH/s
- Intel Iris Xe TGL GT2 (integrated in i5-1135G7): 0.67 MH/s
- Apple M4: 1.99 MH/s

Feel free to add your results and report hardware-specific issues!

## Credits
- [nano-vanity](https://github.com/PlasmaPower/nano-vanity) for ed25519 OpenCL kernel
- [opencl_brute](https://github.com/bkerler/opencl_brute) for sha512 OpenCL kernel
