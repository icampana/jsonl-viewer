# Changelog

## [0.5.0](https://github.com/icampana/jsonl-viewer/compare/jsonl-viewer-v0.4.0...jsonl-viewer-v0.5.0) (2026-02-10)


### Features

* add column sorting functionality and backend support ([f27be42](https://github.com/icampana/jsonl-viewer/commit/f27be420a3241239f835b66d13a2acef6df78413))
* add comprehensive analysis document for FileViewer component and data flow ([414c791](https://github.com/icampana/jsonl-viewer/commit/414c791d3650970c1600736f4054a6969cd99064))
* add sorting interfaces for enhanced data organization ([c36bf53](https://github.com/icampana/jsonl-viewer/commit/c36bf5306998d2c22f7de730f0097946c1110130))
* enhance date parsing to support timezone-less formats as UTC ([e06b60d](https://github.com/icampana/jsonl-viewer/commit/e06b60dd54c1cbc48b9d81ca3dfda02dc89de350))
* implement sorting functionality and enhance data handling in FileViewer and StatsDialog components ([d9dea5f](https://github.com/icampana/jsonl-viewer/commit/d9dea5febbba6268bcee9726ba056a896b568e58))
* implement sorting functionality for JSON lines and search results ([997856a](https://github.com/icampana/jsonl-viewer/commit/997856a8634b0dc81bd902963948fce2234a57ba))
* improve sorting by handling data chunks incrementally for file lines and search results ([14a297b](https://github.com/icampana/jsonl-viewer/commit/14a297b7146e1718d26be5704071b15aac5c75b4))
* optimize sorting performance by pre-extracting sort keys for file lines and search results ([ab998ba](https://github.com/icampana/jsonl-viewer/commit/ab998ba27a3255f8567f39e236d00ec1174a3e68))
* refactor sorting functions to streamline JSON lines and search results processing ([c324dee](https://github.com/icampana/jsonl-viewer/commit/c324deeebb77bba4c69ac4dfc8b214dfc3d0e0db))

## [Unreleased]

### Bug Fixes

* **sort:** Fixed date parsing to support timezone-less formats (e.g., "2024-01-15 10:30:00") by treating them as UTC

## [0.4.0](https://github.com/icampana/jsonl-viewer/compare/jsonl-viewer-v0.3.0...jsonl-viewer-v0.4.0) (2025-12-15)


### Features

* Added basic statistics for the numeric fields on the dataset ([374da96](https://github.com/icampana/jsonl-viewer/commit/374da96d70798454d90cca7227a9be2a4bf1d78b))
* **search:** Added support for quick searches directly from the json tree elements ([acab727](https://github.com/icampana/jsonl-viewer/commit/acab727e10d12c7a826fddfc8bc4fea6fa6816a9))


### Bug Fixes

* small maintenance, removed unused elements ([3aa7fb6](https://github.com/icampana/jsonl-viewer/commit/3aa7fb62b007ca7d1e981633cd5d970ab0b6451f))

## [0.3.0](https://github.com/icampana/jsonl-viewer/compare/jsonl-viewer-v0.2.0...jsonl-viewer-v0.3.0) (2025-12-15)


### Features

* Added smart formatting for nested data objects ([ac4d3a6](https://github.com/icampana/jsonl-viewer/commit/ac4d3a66f16cbe154a91eb6257f97755f565e20b))


### Bug Fixes

* **FileViewer:** allow having up to 100 visible columns by default ([a7ed86b](https://github.com/icampana/jsonl-viewer/commit/a7ed86bb715dcc3445ef8821c8da1e4e364275dd))

## [0.2.0](https://github.com/icampana/jsonl-viewer/compare/jsonl-viewer-v0.1.0...jsonl-viewer-v0.2.0) (2025-12-13)


### Features

* **search:** Solved the issues with the jsonPath search ([e5596ec](https://github.com/icampana/jsonl-viewer/commit/e5596ece999f68686ea49857747d8b0d8d8fc518))


### Bug Fixes

* disabled the regex search option ([90f5bc3](https://github.com/icampana/jsonl-viewer/commit/90f5bc3efa8c334769a5e94a0073f7dede79fab6))

## [0.1.0](https://github.com/icampana/jsonl-viewer/compare/jsonl-viewer-v0.0.1...jsonl-viewer-v0.1.0) (2025-12-13)


### Features

* add GitHub Actions workflow for automated multi-platform Tauri releases. ([c633725](https://github.com/icampana/jsonl-viewer/commit/c633725e2a615ad259e39350aa4b3e3d54715f39))
* added drag and drop support ([101811f](https://github.com/icampana/jsonl-viewer/commit/101811f9fd3eb1f33d2ae3e80db12613bfe19058))
* added support to open files from an URL directly ([7fefd22](https://github.com/icampana/jsonl-viewer/commit/7fefd223e05ca84b8f2c1b486e224cb9a72d9ace))


### Bug Fixes

* added json validator to prevent  other type of files to be loaded ([d2369c9](https://github.com/icampana/jsonl-viewer/commit/d2369c95f9471f2204e5486f1120b86801d3868d))
* corrected search handling ([91708d2](https://github.com/icampana/jsonl-viewer/commit/91708d2f7ff9736f1764ad3a9fdfa05331998c34))
* enabled support for regular jsons in multiline format ([66b9fed](https://github.com/icampana/jsonl-viewer/commit/66b9fed35537fc3b853ab840c3b47db908e5ff6a))
