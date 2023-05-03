# Changelog

All notable changes to this project will be documented in this file. See [commit-and-tag-version](https://github.com/absolute-version/commit-and-tag-version) for commit guidelines.

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