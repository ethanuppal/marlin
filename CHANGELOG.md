# Changelog

## [0.5.0](https://github.com/ethanuppal/marlin/compare/v0.4.0...v0.5.0) (2025-03-24)


### Features

* **verilator:** VCD tracing 🔥 ([#113](https://github.com/ethanuppal/marlin/issues/113)) ([f3ad49a](https://github.com/ethanuppal/marlin/commit/f3ad49a6e0fe10dda25b04f88364331f647f7591))

## [0.4.0](https://github.com/ethanuppal/marlin/compare/v0.3.4...v0.4.0) (2025-03-18)


### Features

* **verilog:** Support `bool` and distinguish signed/unsigned integers over DPI ([#110](https://github.com/ethanuppal/marlin/issues/110)) ([da3596f](https://github.com/ethanuppal/marlin/commit/da3596f0d46fd74a3e536f99a5463e26a7dfb635))

## [0.3.4](https://github.com/ethanuppal/marlin/compare/v0.3.3...v0.3.4) (2025-03-11)


### Bug Fixes

* **verilator:** Only log blocking message when actually blocking ([#105](https://github.com/ethanuppal/marlin/issues/105)) ([4f0a007](https://github.com/ethanuppal/marlin/commit/4f0a007e6e96ce22b350cbceffa58b9f5fb7101a))

## [0.3.3](https://github.com/ethanuppal/marlin/compare/v0.3.2...v0.3.3) (2025-03-11)


### Bug Fixes

* **verilator:** Only log compile messages when actually rebuilding ([#103](https://github.com/ethanuppal/marlin/issues/103)) ([aae7f4d](https://github.com/ethanuppal/marlin/commit/aae7f4d250eebf34c39570eed7fd9bebd3db62f4))

## [0.3.2](https://github.com/ethanuppal/marlin/compare/v0.3.1...v0.3.2) (2025-03-08)


### Bug Fixes

* **verilator:** Support multiple models from one runtime ([#100](https://github.com/ethanuppal/marlin/issues/100)) ([f3c17d1](https://github.com/ethanuppal/marlin/commit/f3c17d16c4cf73b5b54dedf177fc8095d3257379))

## [0.3.1](https://github.com/ethanuppal/marlin/compare/v0.3.0...v0.3.1) (2025-03-08)


### Bug Fixes

* **spade:** Don't `swim build` by default ([#96](https://github.com/ethanuppal/marlin/issues/96)) ([f5adc52](https://github.com/ethanuppal/marlin/commit/f5adc520870ae187dca5b5dfb4992be9e8931444))

## [0.3.0](https://github.com/ethanuppal/marlin/compare/v0.2.1...v0.3.0) (2025-03-06)


### Features

* **veryl:** Initial Veryl support ([#57](https://github.com/ethanuppal/marlin/issues/57)) ([7290f17](https://github.com/ethanuppal/marlin/commit/7290f173f0afe9758e28ff955c38cf0473ce37ed))

## [0.2.1](https://github.com/ethanuppal/marlin/compare/v0.2.0...v0.2.1) (2025-02-24)


### Bug Fixes

* **verilator:** Correctly lock build directory for parallel correctness ([#81](https://github.com/ethanuppal/marlin/issues/81)) ([99949ad](https://github.com/ethanuppal/marlin/commit/99949ad81f32bc99649cb7d1462a703590869ffe))

## [0.2.0](https://github.com/ethanuppal/marlin/compare/v0.1.0...v0.2.0) (2025-02-21)


### Features

* **spade:** Parse `swim.toml` to determine location of simulation Verilog ([#62](https://github.com/ethanuppal/marlin/issues/62)) ([06a9f8f](https://github.com/ethanuppal/marlin/commit/06a9f8f190ec06919ba20747e5bb38da377d1f03))
* **tooling:** Add `swim-marlin` Swim subcommand ([#55](https://github.com/ethanuppal/marlin/issues/55)) ([7f6cb94](https://github.com/ethanuppal/marlin/commit/7f6cb94d69aa9ebc2247f8ae8b75d1b6eae67576))
* **verilator:** Allow disabling Verilator warnings ([#63](https://github.com/ethanuppal/marlin/issues/63)) ([eb14e98](https://github.com/ethanuppal/marlin/commit/eb14e988d8844a8da739c6771a9895d4517cad44))
* **verilator:** Enable multithreading via file locks ([#64](https://github.com/ethanuppal/marlin/issues/64)) ([909516e](https://github.com/ethanuppal/marlin/commit/909516e04057ca99b4c0279a0fe1d00f5e11cadc))
* **verilator:** Show compilation messages during tests ([#67](https://github.com/ethanuppal/marlin/issues/67)) ([4c90169](https://github.com/ethanuppal/marlin/commit/4c9016969fa70ee077d8c3b730f0eed2dbf777a3))
* **verilator:** Support Verilog search paths ([#62](https://github.com/ethanuppal/marlin/issues/62)) ([06a9f8f](https://github.com/ethanuppal/marlin/commit/06a9f8f190ec06919ba20747e5bb38da377d1f03))

## [0.1.0](https://github.com/ethanuppal/marlin/compare/v0.1.0...v0.1.0) (2025-02-09)


### Features

* Abstract away verilator runtime and automate integration ([206e7a5](https://github.com/ethanuppal/marlin/commit/206e7a5eaa40ad37dfbef9198950b6e635d11962))
* Add inline vtable to generated structs ([58ace49](https://github.com/ethanuppal/marlin/commit/58ace49722abd5bc37417b38391d863aa555e2ef))
* Allow specifying clock and reset ports ([7bdd2e0](https://github.com/ethanuppal/marlin/commit/7bdd2e00433b0fff99788de2479c897559368fa2))
* Build verilated top dynamically ([10c4ee2](https://github.com/ethanuppal/marlin/commit/10c4ee2e180aa772bf0f0429e2b9bfc91aede7f0))
* Check for valid source paths ([c5407a4](https://github.com/ethanuppal/marlin/commit/c5407a4a5b512a5b012e5d8ddd085824c5d48b2a))
* Improve #[spade] to parse Spade files directly ([eb9422e](https://github.com/ethanuppal/marlin/commit/eb9422e7967c4832f429fd882b5507f9d2150e3c))
* Initial ([ae8fa9b](https://github.com/ethanuppal/marlin/commit/ae8fa9b8fbdbfd18f9018e35a6046562aff23139))
* **spade:** Allow configuring the `swim` executable path ([#15](https://github.com/ethanuppal/marlin/issues/15)) ([b451162](https://github.com/ethanuppal/marlin/commit/b45116289548bb0082f5d639f7d980fd58a07177))
* **spade:** Improve docs and API ([#11](https://github.com/ethanuppal/marlin/issues/11)) ([76ebe03](https://github.com/ethanuppal/marlin/commit/76ebe036494ec3e6151897d1ac6869ef454eb1e3))
* **spade:** Support `#[no_mangle(all)]` as an alternative ([#31](https://github.com/ethanuppal/marlin/issues/31)) ([a248353](https://github.com/ethanuppal/marlin/commit/a2483531924d125fddc3c6b96f06375ec62c632a))
* Start procedural macro for Verilog interface ([5b804c8](https://github.com/ethanuppal/marlin/commit/5b804c8757f3a2c080ec3b161c595b0c699cedf9))
* Start work on #[spade] macro ([45765e9](https://github.com/ethanuppal/marlin/commit/45765e90e5aa2d20475a3c8dab628a21f3bcff70))
* **verilator:** Revamp Rust DPI to be more usable and powerful ([#48](https://github.com/ethanuppal/marlin/issues/48)) ([8a39a19](https://github.com/ethanuppal/marlin/commit/8a39a197b99f798e1ddbea27e9dff04a11e73c8c))
* **verilator:** Support dynamically-created model bindings ([#32](https://github.com/ethanuppal/marlin/issues/32)) ([a11fff2](https://github.com/ethanuppal/marlin/commit/a11fff2b54092e556da56e198ed768aa6f39d0cc))
* **verilator:** Support Rust DPI with Verilator ([#27](https://github.com/ethanuppal/marlin/issues/27)) ([f1ea8f5](https://github.com/ethanuppal/marlin/commit/f1ea8f592723f691f69c0342403caabce7635aec))


### Bug Fixes

* Correctly determine argument dimensions ([0168ce1](https://github.com/ethanuppal/marlin/commit/0168ce127a312cb683e22ab2db5b431840c4b47b))
* **docs:** Correct file paths in DPI tutorial ([#40](https://github.com/ethanuppal/marlin/issues/40)) ([0786cdd](https://github.com/ethanuppal/marlin/commit/0786cdd1d63f36d22ed140878a6db454e5263a5c))
* **docs:** Correct imports for dynamic models ([#33](https://github.com/ethanuppal/marlin/issues/33)) ([c757e03](https://github.com/ethanuppal/marlin/commit/c757e034dfdf087114f74faffd281b065912c6fd))
* **macros:** Support visibility modifiers on model `struct`s ([#46](https://github.com/ethanuppal/marlin/issues/46)) ([f34c554](https://github.com/ethanuppal/marlin/commit/f34c554a5afee5b1b081ecac2ab1bcf786c3ba9e))
* Prevent same-top different-file collisions ([beb28f3](https://github.com/ethanuppal/marlin/commit/beb28f3af8ed562bd9e57aac25d867d3e3e769b9))
* **spade:** Hotfix for logos version mismatch ([#28](https://github.com/ethanuppal/marlin/issues/28)) ([a486e91](https://github.com/ethanuppal/marlin/commit/a486e91efd7a09972bd30efb5e2d3f20ca2c30a7))


### Performance Improvements

* **verilator:** Only rebuild if source files changed ([#13](https://github.com/ethanuppal/marlin/issues/13)) ([4157f41](https://github.com/ethanuppal/marlin/commit/4157f41fc9430130f78ca21a2adf181e78fc8e72))
