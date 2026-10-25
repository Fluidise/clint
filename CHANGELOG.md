# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### 📚 Documentation

- **changelog:** adopt official git-cliff format and add CHANGELOG.md ([#38](https://github.com/Fluidise/clint/pull/38)) ([f1f07b3](https://github.com/Fluidise/clint/commit/f1f07b383a239ce18d26928fd21c56f8210c4b07))
- **changelog:** align git-cliff formatting and emojis with relay ([#39](https://github.com/Fluidise/clint/pull/39)) ([030114e](https://github.com/Fluidise/clint/commit/030114eca16b663bbd97d68574215ab9687b31d9))

### 👥 Contributors

* @Acrazie

## [0.1.0] - 2026-10-25

### 🚀 Features

- implement agent-first commit message linting ([d18ac3e](https://github.com/Fluidise/clint/commit/d18ac3e890310cc170904fdcefc306c391757366))
- [**breaking**] require explicit policy selection ([#15](https://github.com/Fluidise/clint/pull/15)) ([1159384](https://github.com/Fluidise/clint/commit/115938408cdd990d89a020e5dfd076a8f5c08064))

### 🐛 Bug Fixes

- **ci:** skip auto-assigning bot and app authors ([#36](https://github.com/Fluidise/clint/pull/36)) ([2d638e5](https://github.com/Fluidise/clint/commit/2d638e5ccf7917bddd9578a71df2019ebbcc8b6a))

### ⚙️ Miscellaneous Tasks

- initialize github governance, direct delivery policy, and dogfooding hooks ([97d9118](https://github.com/Fluidise/clint/commit/97d911848d536954e0fa643f8c45b2e8893d6b53))
- configure declarative rust lints and release profile optimizations ([#2](https://github.com/Fluidise/clint/pull/2)) ([8631612](https://github.com/Fluidise/clint/commit/8631612a5d99bd980f5c0670f9d7845539bd021c))
- upgrade Rust toolchain and MSRV to 1.99.1 ([#8](https://github.com/Fluidise/clint/pull/8)) ([6a00c1b](https://github.com/Fluidise/clint/commit/6a00c1b4e50b200e9e9662d88a42a6adeefe9c7e))
- modernize quality checks, property testing, coverage and release automation ([#21](https://github.com/Fluidise/clint/pull/21)) ([02bea61](https://github.com/Fluidise/clint/commit/02bea616b186bc90f6296ce4e38eaaa4bcf46dce))
- **deps:** update dependency python to 3.15 ([#24](https://github.com/Fluidise/clint/pull/24)) ([cb49f2f](https://github.com/Fluidise/clint/commit/cb49f2fabd07a28e657a20bf525349695c08e8c5))
- **deps:** update actions/checkout action to v7 ([#29](https://github.com/Fluidise/clint/pull/29)) ([a7cc6f3](https://github.com/Fluidise/clint/commit/a7cc6f355fb57327ff18dc27cacb60946ca6e549))
- **deps:** update node.js to v26.12.1 ([#28](https://github.com/Fluidise/clint/pull/28)) ([85386c1](https://github.com/Fluidise/clint/commit/85386c1e15ad0fdc25133d7e2aa4347eb271e138))
- **deps:** update dependency wcwidth to v0.9.4 ([#30](https://github.com/Fluidise/clint/pull/30)) ([fd313ff](https://github.com/Fluidise/clint/commit/fd313ff38991255551052d28d5f505b28d633afc))
- **deps:** update actions/setup-python action to v7 ([#31](https://github.com/Fluidise/clint/pull/31)) ([a340798](https://github.com/Fluidise/clint/commit/a3407987ad13ffeff7018a89b076a13c9add97ce))
- **deps:** update dependency deprecated to v3 ([#32](https://github.com/Fluidise/clint/pull/32)) ([fce6e3c](https://github.com/Fluidise/clint/commit/fce6e3c075d321acb9132a91406d74ae000d9b29))
- **deps:** update github artifact actions ([#33](https://github.com/Fluidise/clint/pull/33)) ([b3f3cc0](https://github.com/Fluidise/clint/commit/b3f3cc0da38e96f8113ad7cf5b304dfafc073573))
- prepare unsigned 0.1.0 release ([#27](https://github.com/Fluidise/clint/pull/27)) ([9939ae3](https://github.com/Fluidise/clint/commit/9939ae33d2a253b2db1a9bb0434c925521262b9e))
- **deps:** update actions/checkout action to v7 ([#34](https://github.com/Fluidise/clint/pull/34)) ([9ca8764](https://github.com/Fluidise/clint/commit/9ca8764eb1b685cb42fb98df43878f455f9e699c))
- **deps:** update actions/setup-python action to v7 ([#35](https://github.com/Fluidise/clint/pull/35)) ([e700754](https://github.com/Fluidise/clint/commit/e7007544cac2251301bf3db2d16215d93afe79c4))
- **distribution:** authorize public distribution and finalize 0.1.0 runtime review ([#37](https://github.com/Fluidise/clint/pull/37)) ([93766c6](https://github.com/Fluidise/clint/commit/93766c607b105033bfe5aa26fd8795b93ad338e8))

### ⚡ Performance

- validate commit messages without a per-line index ([#12](https://github.com/Fluidise/clint/pull/12)) ([9e108cb](https://github.com/Fluidise/clint/commit/9e108cbe35c99f948361d606f404ac1bffcdadbe))
- **hooks:** avoid redundant Git root discovery ([#14](https://github.com/Fluidise/clint/pull/14)) ([17e8aca](https://github.com/Fluidise/clint/commit/17e8aca8a6b5b4b0db4a6e56f3f0c99d56511a4e))
- **hooks:** avoid redundant Git root discovery ([#14](https://github.com/Fluidise/clint/pull/14)) ([5861192](https://github.com/Fluidise/clint/commit/586119283e978899278a593bc1ff81f77971b2bd))
- optimize ReDoS scanning, JSON output streaming and input buffering ([#17](https://github.com/Fluidise/clint/pull/17)) ([888b5a5](https://github.com/Fluidise/clint/commit/888b5a5b98d938b7eb5ddab889359eee9aa2339c))
- eliminate startup allocations, optimize stdin buffering and json fast-path ([#20](https://github.com/Fluidise/clint/pull/20)) ([6e0f3cd](https://github.com/Fluidise/clint/commit/6e0f3cddbf88cd6824f7b53f9ab385f13ccb0dcf))

### 🔄 Refactor

- rename project to fluidise-clint ([0b9ea22](https://github.com/Fluidise/clint/commit/0b9ea22f94fb962188e3046642898d44e1f7591e))

### 📚 Documentation

- add comprehensive project README ([65f704b](https://github.com/Fluidise/clint/commit/65f704b0a13478e6d152565d7d0403042590f111))
- update GitHub repository references to Fluidise/clint ([405133d](https://github.com/Fluidise/clint/commit/405133dbe2b99d4185c4f93fde3e4c087c22c8d6))
- expand comprehensive bun-style benchmark suite across 5 industry linters ([#5](https://github.com/Fluidise/clint/pull/5)) ([5c7c5bf](https://github.com/Fluidise/clint/commit/5c7c5bfe26ee879e1fa8a71cc8d5d3146ef2398b))
- center README header and style badges with banner palette ([#6](https://github.com/Fluidise/clint/pull/6)) ([d999408](https://github.com/Fluidise/clint/commit/d999408a2536fe3edb19a20ecfea9428535c52cb))
- record ADR 0001 on competitor feature gaps and evolution tracks ([#7](https://github.com/Fluidise/clint/pull/7)) ([a03f43c](https://github.com/Fluidise/clint/commit/a03f43c9dced3041164df1800f969d4a6436320f))
- update README benchmarks with zero-allocation lazy stream results ([#16](https://github.com/Fluidise/clint/pull/16)) ([53db2fb](https://github.com/Fluidise/clint/commit/53db2fb05c3ba92c6f8c935af372977fe074f09b))
- update README benchmarks with latest engine and throughput gains ([#18](https://github.com/Fluidise/clint/pull/18)) ([84f77de](https://github.com/Fluidise/clint/commit/84f77dea947839da8ca70c0d47dda68bbfcb9c80))

### 🧪 Testing

- validate benchmark correctness and comparison contracts ([#19](https://github.com/Fluidise/clint/pull/19)) ([d0940c1](https://github.com/Fluidise/clint/commit/d0940c1e7bfb04fde7e6561beba9691da6a8660d))

### 👷 Continuous Integration

- add GitHub Actions workflow and update gitignore ([40c4df6](https://github.com/Fluidise/clint/commit/40c4df6024bbdb97fc537894ce7b63fc8b860649))
- configure automated PR author assignment and conventional labels triage ([#3](https://github.com/Fluidise/clint/pull/3)) ([f8e4911](https://github.com/Fluidise/clint/commit/f8e49114dad6b8b4ac0971a0c7101a1087ba4ac9))
- optimize pipeline efficiency and add multi-os test matrix ([#4](https://github.com/Fluidise/clint/pull/4)) ([c5dc3fb](https://github.com/Fluidise/clint/commit/c5dc3fb4a51d6d06a63aa337b2e2589ab921f121))
- prepare candidate binary distribution ([#9](https://github.com/Fluidise/clint/pull/9)) ([672a046](https://github.com/Fluidise/clint/commit/672a046818cedd6423fafe0ecab2dee778677f14))
- harden validation and synchronize PR labels ([#10](https://github.com/Fluidise/clint/pull/10)) ([8d7aad2](https://github.com/Fluidise/clint/commit/8d7aad279283904451af87a0f611a3920dcd4499))
- add auto-merge job for validated pull requests ([#11](https://github.com/Fluidise/clint/pull/11)) ([4f60848](https://github.com/Fluidise/clint/commit/4f60848674b49a4c8e6bda943eeb02dfd3a6e979))
- skip CI checks on draft pull requests ([#13](https://github.com/Fluidise/clint/pull/13)) ([c742e55](https://github.com/Fluidise/clint/commit/c742e55f7b2d935fd31ff0fb5efd4166d0f1a178))
- configure renovate bot for dependency and action updates ([#26](https://github.com/Fluidise/clint/pull/26)) ([25e2bfc](https://github.com/Fluidise/clint/commit/25e2bfc1bdfcd55ff0c3281d3667817d653e17e6))


### ❤️ New Contributors

* @Acrazie made their first contribution in [#37](https://github.com/Fluidise/clint/pull/37)


### 👥 Contributors

* @Acrazie


[unreleased]: https://github.com/Fluidise/clint/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Fluidise/clint/tree/v0.1.0

<!-- generated by git-cliff -->
