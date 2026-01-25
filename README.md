# firefly-main

The entry point for the [Firefly Zero](https://fireflyzero.com/) device firmware.

It initializes all drivers and [ESP32](https://en.wikipedia.org/wiki/ESP32) peripherals, passes it into [firefly-hal](https://github.com/firefly-zero/firefly-hal), and then runs [firefly-runtime](https://github.com/firefly-zero/firefly-runtime) in a loop.

## Flashing

First, you need to know which version of the device you have:

* `v1`: The very first prototype (FOSDEM 2025). It's in a simple rectangle case and powered by two ESP32-S3.
* `v2`: The next iteration of the prototype. The hardware components are the same but the pinout is different. This is the first prototype in a case with rounded corners.

Flashing:

1. [Install espup](https://github.com/esp-rs/espup)
1. [Install task](https://taskfile.dev/)
1. `espup install`
1. `. ~/export-esp.sh`
1. Connect to the right chip on the device.
1. `task flash -- --port /dev/ttyACM0 --features v2`

## License

[GPL-3.0](./LICENSE). Any forks and extensions for Firefly Zero must be open source. However, this does not affect apps and games running on Firefly Zero. You can make your apps free or paid, open-source or proprietary.
