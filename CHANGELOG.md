# Changelog

## [0.1.1](https://github.com/sousandrei/firesquid/compare/v0.1.0...v0.1.1) (2024-06-13)


### Bug Fixes

* **lint:** clippy ([8125edc](https://github.com/sousandrei/firesquid/commit/8125edc71c2beae0c85f827b94c8d183600f5b9a))

## 0.1.0 (2023-12-17)


### Features

* Adding better logging ([0230dd1](https://github.com/sousandrei/firesquid/commit/0230dd1c2d34ccb186f0f1f07b16194faeb7eb43))
* Adding deb assets and removing old cli params ([371ec50](https://github.com/sousandrei/firesquid/commit/371ec50cc69c8ca3363fe68f55c49b65c7f96bc4))
* Adding log folder initalization ([ebf655c](https://github.com/sousandrei/firesquid/commit/ebf655c8a63ff1fb407cd4bb86d81c119f312f72))
* Adding stdout and stderr logging to file ([20db069](https://github.com/sousandrei/firesquid/commit/20db069fa2ea98594d95c1922be5653f0f6b5f1c))
* **ci:** adding first travis build script ([c206193](https://github.com/sousandrei/firesquid/commit/c206193316c86baf692871c91bc1c7411b0c0380))
* **cp:** process spawning and first vm spawning ([6af1475](https://github.com/sousandrei/firesquid/commit/6af14755a9449118ced93ed48db4997b27480a37))
* Daemon variable to differentiate cli and service ([5ed6576](https://github.com/sousandrei/firesquid/commit/5ed6576d944b12464d15ac4f86df60fba8623618))
* first commit ([5a7058f](https://github.com/sousandrei/firesquid/commit/5a7058fad56b2085d0a9d1763a0cade86140dfd6))
* Gracefully shutdown draft! ([d861129](https://github.com/sousandrei/firesquid/commit/d86112956d776158bbff20f80367a3d4e9607085))
* Implement basic cli-daemon on the same binary ([2396336](https://github.com/sousandrei/firesquid/commit/239633602b3a3a561220bb1f130c896f942c53a6))
* Improving locking mechanism ([f5e658a](https://github.com/sousandrei/firesquid/commit/f5e658adef857e5a000dd26fe2bad4aab899fdac))
* **iops:** adding kernel and drive code ([f001a19](https://github.com/sousandrei/firesquid/commit/f001a1901311e3a075fc291638f61acf1887d8e5))
* Logging the error when making request to vms ([cad021a](https://github.com/sousandrei/firesquid/commit/cad021a8800ff75466981b1934fa8566781bb105))
* Making state a pointer and vms a Vec ([4df3f0b](https://github.com/sousandrei/firesquid/commit/4df3f0b8875a9a393e5777ce30c47cf7a365f728))
* moving to http using unix socket ([acb8590](https://github.com/sousandrei/firesquid/commit/acb8590c97147c4e6d027fc33d553f3ffe0b6ad7))
* Starting work on daemon process ([c269c9d](https://github.com/sousandrei/firesquid/commit/c269c9d24e1d62ed3749e531908e25cb484d9ba2))
* Testing gracefully shutting down hyper server ([23a4c37](https://github.com/sousandrei/firesquid/commit/23a4c37f2451ec5b26661ceb81ff34b77278c41f))
* **tmp:** adding first draft of tmp folder management ([9502d52](https://github.com/sousandrei/firesquid/commit/9502d52a7b3ad9843048f6ba51b451e8a33f561b))


### Bug Fixes

* Add sleep to wait for firecracker socket ([e68e14e](https://github.com/sousandrei/firesquid/commit/e68e14e5aa4c1fd7535f2d7c2d2b79b85a69d37b))
* Aligning permissions on deb files ([0d94a65](https://github.com/sousandrei/firesquid/commit/0d94a65354bb6f9fb78704d590f8bdda7f8ed38c))
* **clippy:** trait bounds and clippy warnings ([#27](https://github.com/sousandrei/firesquid/issues/27)) ([2496d84](https://github.com/sousandrei/firesquid/commit/2496d845bca0b06e4b8c9afe91f3010c13253582))
* Fix headers in requests to firecracker ([6010f31](https://github.com/sousandrei/firesquid/commit/6010f31200393a90ab4359fb131e941f6b54a6ed))
* Fix response body: from string to json ([bd4a9b2](https://github.com/sousandrei/firesquid/commit/bd4a9b28ceb8f2ab045e4546f5c91d5c9cafdbe9))
* Fixing git ignore ([afb0be3](https://github.com/sousandrei/firesquid/commit/afb0be3ce93fec69976e55c35479ca6e8fe8162f))
* **iops:** remove error when trying to delete file ([17d0008](https://github.com/sousandrei/firesquid/commit/17d000829ac8c9f7900577683ff0703a3193463a))
* machine_name arg only when needed ([231fe66](https://github.com/sousandrei/firesquid/commit/231fe66935c6d02b8e6f1d56c2c87b33865de047))
* **travis:** creating dummy kernel file for tests ([7bf20a8](https://github.com/sousandrei/firesquid/commit/7bf20a8cab1a164f9ed059c9b7e37af900deaeff))
