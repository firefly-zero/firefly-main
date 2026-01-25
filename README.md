# firefly-main

The entry point for the [Firefly Zero](https://fireflyzero.com/) device firmware.

It initializes all drivers and [ESP32](https://en.wikipedia.org/wiki/ESP32) peripherals, passes it into [firefly-hal](https://github.com/firefly-zero/firefly-hal), and then runs [firefly-runtime](https://github.com/firefly-zero/firefly-runtime) in a loop.

## Flashing

First, you need to know which version of the device you have:

* `v1`: The very first prototype (FOSDEM 2025). It's in a simple rectangle case and powered by two ESP32-S3.
* `v2`: The next iteration of the prototype. The hardware components are the same but the pinout is different. This is the first prototype in a case with rounded corners.

Flashing:

1. Clone the repo.
1. Install [task](https://taskfile.dev/docs/installation).
1. Install [espup and xtensa fork of rust](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html).
1. Connect the device via USB. Make sure you've connected to the right chip (main, not IO).
1. `task flash -- --port /dev/ttyACM0 --features v2`

## License

[GPL-3.0](./LICENSE). Any forks and extensions for Firefly Zero must be open source. However, this does not affect apps and games running on Firefly Zero. You can make your apps free or paid, open-source or proprietary.
