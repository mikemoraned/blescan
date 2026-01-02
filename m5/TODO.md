* [x] get something installed on m5 device:
    * device: https://shop.m5stack.com/products/m5stickc-plus2-esp32-mini-iot-development-kit
    * following https://docs.espressif.com/projects/rust/book/
    * `ESP32-PICO-V3-02` is an Xtensa Device
    * espup install:
    ```
    cargo install espup --locked
    espup install
    ```
    * do https://github.com/esp-rs/espup?tab=readme-ov-file#environment-variables-setup: I added to my ~/.zshrc
    * helpful tools:
    ```
    cargo install esp-generate --locked
    cargo install espflash --locked
    ```
        * debugging (though may not work for my device without further hardware support):
        ```
        brew tap probe-rs/probe-rs
        brew install probe-rs
        probe-rs complete install
        ```
    ```
    cargo install esp-config --features=tui --locked
    ```
    * try a simple project:
    ```
    esp-generate
    ```
        * note: need to remove the `.git` folder it generates
* [x] above seems to be using the lower-level `no_std` approach, so trying a higher-level template
    * https://github.com/esp-rs/esp-idf-template?tab=readme-ov-file#prerequisites
    ```
    cargo install cargo-generate
    cargo install ldproxy
    # espup and espflash already installed
    ```
    * https://github.com/esp-rs/esp-idf-template?tab=readme-ov-file
    ```
    cargo generate esp-rs/esp-idf-template cargo
    ```
* [x] get something showing on display itself
* [x] get the device scanning for bluetooth devices in a main loop and then having the data be (available to be) collected
    * [x] device advertising itself over BLE
    * [x] device repeatedly scanning
* [ ] ...