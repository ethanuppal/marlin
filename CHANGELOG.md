# Changelog

## [0.1.0](https://github.com/ethanuppal/dumbname/compare/v0.1.0...v0.1.0) (2025-02-03)


### Features

* Abstract away verilator runtime and automate integration ([206e7a5](https://github.com/ethanuppal/dumbname/commit/206e7a5eaa40ad37dfbef9198950b6e635d11962))
* Add inline vtable to generated structs ([58ace49](https://github.com/ethanuppal/dumbname/commit/58ace49722abd5bc37417b38391d863aa555e2ef))
* Allow specifying clock and reset ports ([7bdd2e0](https://github.com/ethanuppal/dumbname/commit/7bdd2e00433b0fff99788de2479c897559368fa2))
* Build verilated top dynamically ([10c4ee2](https://github.com/ethanuppal/dumbname/commit/10c4ee2e180aa772bf0f0429e2b9bfc91aede7f0))
* Check for valid source paths ([c5407a4](https://github.com/ethanuppal/dumbname/commit/c5407a4a5b512a5b012e5d8ddd085824c5d48b2a))
* Improve #[spade] to parse Spade files directly ([eb9422e](https://github.com/ethanuppal/dumbname/commit/eb9422e7967c4832f429fd882b5507f9d2150e3c))
* Initial ([ae8fa9b](https://github.com/ethanuppal/dumbname/commit/ae8fa9b8fbdbfd18f9018e35a6046562aff23139))
* **spade:** Allow configuring the `swim` executable path ([#15](https://github.com/ethanuppal/dumbname/issues/15)) ([b451162](https://github.com/ethanuppal/dumbname/commit/b45116289548bb0082f5d639f7d980fd58a07177))
* **spade:** Improve docs and API ([#11](https://github.com/ethanuppal/dumbname/issues/11)) ([76ebe03](https://github.com/ethanuppal/dumbname/commit/76ebe036494ec3e6151897d1ac6869ef454eb1e3))
* Start procedural macro for Verilog interface ([5b804c8](https://github.com/ethanuppal/dumbname/commit/5b804c8757f3a2c080ec3b161c595b0c699cedf9))
* Start work on #[spade] macro ([45765e9](https://github.com/ethanuppal/dumbname/commit/45765e90e5aa2d20475a3c8dab628a21f3bcff70))


### Bug Fixes

* Correctly determine argument dimensions ([0168ce1](https://github.com/ethanuppal/dumbname/commit/0168ce127a312cb683e22ab2db5b431840c4b47b))
* Prevent same-top different-file collisions ([beb28f3](https://github.com/ethanuppal/dumbname/commit/beb28f3af8ed562bd9e57aac25d867d3e3e769b9))
* **spade:** Hotfix for logos version mismatch ([#28](https://github.com/ethanuppal/dumbname/issues/28)) ([a486e91](https://github.com/ethanuppal/dumbname/commit/a486e91efd7a09972bd30efb5e2d3f20ca2c30a7))


### Performance Improvements

* **verilator:** Only rebuild if source files changed ([#13](https://github.com/ethanuppal/dumbname/issues/13)) ([4157f41](https://github.com/ethanuppal/dumbname/commit/4157f41fc9430130f78ca21a2adf181e78fc8e72))
