# Changelog

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
