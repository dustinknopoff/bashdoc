<a name=""></a>

## Bashdoc (2018-12-22)

#### Bug Fixes

- **0.4.0:** ([31d2454b](https://github.com/dustinknopoff/bashdoc/commit/31d2454bcb44a074c2b22e1c58ff38a64f78a830))
- **All:** Apply clippy fixes ([bca0b613](https://github.com/dustinknopoff/bashdoc/commit/bca0b613adc82452bfb3c70cfe15c205d5b74816))
- **Cargo.toml:**
  - version bump for cargo ([b7b5e624](https://github.com/dustinknopoff/bashdoc/commit/b7b5e62481ff4e448a7d49c12321e941beeced6c))
  - Version bump to update cargo ([0bbd48aa](https://github.com/dustinknopoff/bashdoc/commit/0bbd48aab4500b2b6ee81f39c63453a745231db0))
  - Experimental badge ([075a9d68](https://github.com/dustinknopoff/bashdoc/commit/075a9d68be855ced117cf1f60b30629c2b408488))
  - Metadata for publishing added ([8fda35d6](https://github.com/dustinknopoff/bashdoc/commit/8fda35d68637380499b8aaf515570bc7e41cafc3))
  - Remove email ([29d67053](https://github.com/dustinknopoff/bashdoc/commit/29d67053bb62e31458a9be9635b6fdb078f52639))
- **Clippy:** Allow Cyclomatic Complexity on nom parser ([6ce68ec9](https://github.com/dustinknopoff/bashdoc/commit/6ce68ec90e0525977b716ce579a24d64b491ca33))
- **Docs:** Better md formatting ([8d4df95b](https://github.com/dustinknopoff/bashdoc/commit/8d4df95b3759f9ec9472c7322f3229f297fae22f))
- **README:**
  - include cargo install instructions ([5c504667](https://github.com/dustinknopoff/bashdoc/commit/5c5046676c2842a7dbcaedc80f47c1b1a365039c))
  - code blocks should be highlighted for bash not rust ([62db2ad2](https://github.com/dustinknopoff/bashdoc/commit/62db2ad2f253c25517268ff7ead595f4c73f063d))
- **Support windows again:** ([46f6267a](https://github.com/dustinknopoff/bashdoc/commit/46f6267a074f0024c960d29d5e92ce0fd76848a1))
- **Watcher:** Better information on watching files ([fbde071d](https://github.com/dustinknopoff/bashdoc/commit/fbde071d0d578bfa0d64e73984593ab5a200a368))
- **Windows support re-added:** ([ca69c443](https://github.com/dustinknopoff/bashdoc/commit/ca69c443acd035bb6b7a3e6df7a975a9908c4f42))
- **cargo.toml:** Specify files to package ([6d99b502](https://github.com/dustinknopoff/bashdoc/commit/6d99b502cb08b108b80b907d618c4bc13326f528))
- **cli:** Removed requirement for `--location` on all calls ([e93d9144](https://github.com/dustinknopoff/bashdoc/commit/e93d9144979bffe8c36dffc1199cf8805752ee7e))
- **cli.yml:** Keep version numbers consistent ([0347fd80](https://github.com/dustinknopoff/bashdoc/commit/0347fd804d36cd9c39debcd50fb9878d763b96fc))
- **clippy:** implement clippy suggestions ([4f59d771](https://github.com/dustinknopoff/bashdoc/commit/4f59d7713ad4de2abb9188b63c6a2569d678b927))
- **delimiters:** Resolves iisue where user with no BASHDOC_CONFIG_PATH caused a panic ([8738852d](https://github.com/dustinknopoff/bashdoc/commit/8738852d8acc453b3ee1947da3f7b18fdc147a72))
- **demo:** remove old examples ([dea6213a](https://github.com/dustinknopoff/bashdoc/commit/dea6213abb5eec5165504b3bd0dab183e879e2ba))
- **docs.rs:** Remove debug printlns ([50b35b08](https://github.com/dustinknopoff/bashdoc/commit/50b35b08d64c06800359babb5823e7af7e94782f))
- **example:**
  - weird coloring ([bb2be77c](https://github.com/dustinknopoff/bashdoc/commit/bb2be77cfa59527ec595abdc24d035f3f34c048d))
  - fixes ([bd1d76bd](https://github.com/dustinknopoff/bashdoc/commit/bd1d76bdac0fb0ebd9c3ea2d770eadfcb8f19002))
- **header:** Improved header looks ([b8f5433c](https://github.com/dustinknopoff/bashdoc/commit/b8f5433c2100d2172699cc32606b4bd15eaea96c))
- **html:** remove example html to stop inflation of html in repo ([e9965c58](https://github.com/dustinknopoff/bashdoc/commit/e9965c58df218db21b6fe04d05748a10d132d334))
- **screenshot:** ([87d31b24](https://github.com/dustinknopoff/bashdoc/commit/87d31b2469614f92ca1a42fa81d08d590692a610))
- **static:** remove unnecessary CSS files ([38b6f064](https://github.com/dustinknopoff/bashdoc/commit/38b6f06476e8ecaba05c497cda3022bec6fa8d5d))

#### Features

- **Extracted:** Parsing of bashdocs from text now includes the line number of the function extract ([0ce89d6e](https://github.com/dustinknopoff/bashdoc/commit/0ce89d6e65efd4852b79aecf89a37d746922f4bf))
- **Windows:** support for windows filepaths ([fec2de62](https://github.com/dustinknopoff/bashdoc/commit/fec2de6235b82c18ebaf839aa0e736196850ab40))
- **cli.yml:** Can provide a custom template for html documentation ([05936821](https://github.com/dustinknopoff/bashdoc/commit/059368217d8f155662a1ee3e156d0e0e373c2c03), breaks [#](https://github.com/dustinknopoff/bashdoc/issues/))
- **docs.rs:** Upgrade to 2018 edition, move to nom for parsing ([e7b0541b](https://github.com/dustinknopoff/bashdoc/commit/e7b0541b5fe26db23198e15100916f0d59fbeeae))
- **output HTML:**
  - Basic example of outputting html documentation of a bashdoc call ([0155a3e8](https://github.com/dustinknopoff/bashdoc/commit/0155a3e84058b2378b8c0dfc7b37e62c2a3bda7e))
  - Extremely bare example of creating html documentation from a bashdoc call ([030645ef](https://github.com/dustinknopoff/bashdoc/commit/030645ef8fc1c2d66e4095c14146c6ac9f6ec8d7))
- **script.js:**
  - Highlight clicked on function ([5bab275c](https://github.com/dustinknopoff/bashdoc/commit/5bab275cf8fb291934122d7bc9520733530a775b))
  - Highlight clicked on function ([b8d25409](https://github.com/dustinknopoff/bashdoc/commit/b8d25409d328ad1d282ba45c58b3a19f0630166b))

#### Breaking Changes

- **cli.yml:** Can provide a custom template for html documentation ([05936821](https://github.com/dustinknopoff/bashdoc/commit/059368217d8f155662a1ee3e156d0e0e373c2c03), breaks [#](https://github.com/dustinknopoff/bashdoc/issues/))

#### Performance

- **generate_doc_file:** Parallelize Doc generation ([984e546e](https://github.com/dustinknopoff/bashdoc/commit/984e546e19aed5aec4ad91d6cf4b506b03c31d42))
