use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::{SpiDriver, SpiDeviceDriver, SpiConfig, config::DriverConfig};
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::units::Hertz;
use std::thread;
use std::time::Duration;

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use display_interface_spi::SPIInterface;
use mipidsi::Builder;
use mipidsi::options::{ColorOrder, Orientation, Rotation};

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    // Get peripherals
    let peripherals = Peripherals::take().unwrap();

    // GPIO27 controls the backlight on M5StickC Plus2
    let mut backlight = PinDriver::output(peripherals.pins.gpio27).unwrap();

    // Turn backlight on
    backlight.set_high().unwrap();
    log::info!("Backlight turned on");

    // M5StickC Plus2 display pins:
    // MOSI (SDA): GPIO15
    // CLK (SCL): GPIO13
    // CS: GPIO5
    // DC: GPIO14
    // RST: GPIO12

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio13;
    let mosi = peripherals.pins.gpio15;
    let cs = peripherals.pins.gpio5;
    let dc = peripherals.pins.gpio14;
    let rst = peripherals.pins.gpio12;

    // Initialize SPI
    let driver = SpiDriver::new(spi, sclk, mosi, None::<esp_idf_svc::hal::gpio::AnyIOPin>, &DriverConfig::new()).unwrap();

    let config = SpiConfig::new().baudrate(Hertz(26_000_000));
    let spi_device = SpiDeviceDriver::new(driver, Some(cs), &config).unwrap();

    let dc_pin = PinDriver::output(dc).unwrap();
    let rst_pin = PinDriver::output(rst).unwrap();

    // Create display interface
    let di = SPIInterface::new(spi_device, dc_pin);

    // Initialize ST7789 display using mipidsi
    let mut display = Builder::new(mipidsi::models::ST7789, di)
        .display_size(135, 240)
        .reset_pin(rst_pin)
        .color_order(ColorOrder::Rgb)
        .init(&mut Delay::new_default())
        .unwrap();

    // Set orientation to landscape (90 degree rotation)
    display.set_orientation(Orientation::new().rotate(Rotation::Deg90)).unwrap();

    log::info!("Display initialized");

    // Clear display to black
    display.clear(Rgb565::BLACK).unwrap();

    // Get display dimensions (in landscape: 240x135)
    let width = 240;
    let height = 135;

    // Calculate center and radius
    let center_x = width / 2;
    let center_y = height / 2;
    let radius = (width.min(height) / 2) as u32;

    log::info!("Drawing circle at ({}, {}) with radius {}", center_x, center_y, radius);

    // Draw circle in the middle of the screen
    let circle = Circle::with_center(
        Point::new(center_x as i32, center_y as i32),
        radius
    );

    circle.into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 2))
        .draw(&mut display)
        .unwrap();

    log::info!("Circle drawn");

    // Keep the display on
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
