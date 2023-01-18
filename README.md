# Piano Visualizer
### Oddly-made led-visualization software for my piano

This is very much WIP. Feel free to submit PRs to fix ugly code or whatever, since I am pretty new to Rust.

## :warning: WARNING: This is a mess to cross compile !

### Cross-compiling

You will first need a linker for `arm-unknown-gnueabi`, you can find many on [Github](https://github.com/search?q=toolchain+arm) for your host OS.

You will also need to retrieve the following libraries via for example a docker container with the same arch as the target:

- `libportmidi.so`
- `libusb.so`
- `libasound.so.2`

and any library that the previous depend on.

These libraries must be placed most of the time at `/path/to/arm-unknown-linux-gnueabi/arm-unknown-linux-gnueabi/sysroot/lib`

You can then specify your linker in the `.cargo/config.toml` file:

```yaml
[target.arm-unknown-linux-gnueabi]
linker = "/path/to/arm-unknown-linux-gnueabi/bin/arm-unknown-linux-gnueabi-gcc"
```

There are scripts included to make your life easier when you want to deploy to a Raspberry-PI for example.

- `deploy.py`: Build, deploy and run a Rust program on a remote machine
- `env.sh`: Run this script before building. Sets the required environment variables for cargo etc.

### Web UI

The web ui is very very very very simple. It just allows you to interact with the REST API provided by the Rust program (eg. change the color mode)
