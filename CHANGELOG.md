# Changelog

All notable changes to this project will be documented in this file. See [commit-and-tag-version](https://github.com/absolute-version/commit-and-tag-version) for commit guidelines.

## [0.3.0](https://github.com/DCsunset/snapshot-fuse/compare/v0.2.0...v0.3.0) (2023-05-13)


### Features

* add garbage collection ([b927484](https://github.com/DCsunset/snapshot-fuse/commit/b92748492340b851d4fc360ecfbae044d78c0ce0))
* supoprt setting timeout for metadata ([d55fd03](https://github.com/DCsunset/snapshot-fuse/commit/d55fd03278ab8598ae446d41f45fcf9e30c00c8d))
* update inode info only when outdated ([36820df](https://github.com/DCsunset/snapshot-fuse/commit/36820dfda2402e2871e3163c7d990e74d27d0fe3))

## [0.2.0](https://github.com/DCsunset/snapshot-fuse/compare/v0.1.1...v0.2.0) (2023-05-04)


### Features

* support pax format for long file names ([2381ac0](https://github.com/DCsunset/snapshot-fuse/commit/2381ac093270033d9e23f580102bd3e39a01e75f))


### Bug Fixes

* add option to enable auto unmount ([4c09949](https://github.com/DCsunset/snapshot-fuse/commit/4c09949fb6bf26ddebd70b2adf66e450f212b792))

## [0.1.1](https://github.com/DCsunset/snapshot-fuse/compare/v0.1.0...v0.1.1) (2023-05-03)


### Bug Fixes

* auto unmount when program terminates ([c5e4955](https://github.com/DCsunset/snapshot-fuse/commit/c5e4955095d326986b927ed2b136d5ff5bb7fa18))

## 0.1.0 (2023-05-03)


### Features

* add basic functionality for mounting ([00417b7](https://github.com/DCsunset/snapshot-fuse/commit/00417b745693b22e058b13fefcb5908e1187acb1))
* add tar extension to each file ([d1c8a49](https://github.com/DCsunset/snapshot-fuse/commit/d1c8a49857c830d2e50f7affe589eb14cfa7efa5))
* show statfs from original fs ([972aae2](https://github.com/DCsunset/snapshot-fuse/commit/972aae295f08efd99ad919cfaaacc1d28df2a9c6))
* support archive individual files in root dir ([42cc854](https://github.com/DCsunset/snapshot-fuse/commit/42cc85489735b04a3aae38e0d03a241c93aee46d))
* support more cmd line options ([5858c1c](https://github.com/DCsunset/snapshot-fuse/commit/5858c1ca14b05250c92077d5e9aab7f58784be30))
* support random access for virtual tar files ([a5eb5ae](https://github.com/DCsunset/snapshot-fuse/commit/a5eb5ae48c0bd085cd5582046b16dad24bfcacfb))


### Bug Fixes

* fix attr derivation ([4a61ae7](https://github.com/DCsunset/snapshot-fuse/commit/4a61ae78818def7062c17920781df6f728e11d2a))
* remove libfuse feature to solve dependency issue in ci ([1835cb9](https://github.com/DCsunset/snapshot-fuse/commit/1835cb9ce8fba510878efffed8bbfefc6dd4876e))
* strip source dir prefix and handle dir correctly ([8b10445](https://github.com/DCsunset/snapshot-fuse/commit/8b10445a79d60221bda25ad4c707a55303aecf7f))
