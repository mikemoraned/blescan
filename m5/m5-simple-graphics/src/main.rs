use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::{SpiDriver, SpiDeviceDriver, SpiConfig, config::DriverConfig};
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::units::Hertz;
use std::thread;
use std::time::{Duration, Instant};

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use display_interface_spi::SPIInterface;
use mipidsi::Builder;
use mipidsi::options::{ColorOrder, ColorInversion};

// M5StickC Plus2 display constants
const DISPLAY_WIDTH: u32 = 135;
const DISPLAY_HEIGHT: u32 = 240;
const DISPLAY_OFFSET_X: u16 = 52;
const DISPLAY_OFFSET_Y: u16 = 40;
const SPI_BAUDRATE: u32 = 26_000_000;

// Animation constants
const CYCLE_DURATION_MS: u32 = 2500;
const FRAME_DELAY_MS: u64 = 50;
const CIRCLE_STROKE_WIDTH: u32 = 4;

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

    let config = SpiConfig::new().baudrate(Hertz(SPI_BAUDRATE));
    let spi_device = SpiDeviceDriver::new(driver, Some(cs), &config).unwrap();

    let dc_pin = PinDriver::output(dc).unwrap();
    let rst_pin = PinDriver::output(rst).unwrap();

    // Create display interface
    let di = SPIInterface::new(spi_device, dc_pin);

    // Initialize ST7789 display using mipidsi
    // M5StickC Plus2 requires both RGB order and color inversion
    let mut display = Builder::new(mipidsi::models::ST7789, di)
        .display_size(DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16)
        .display_offset(DISPLAY_OFFSET_X, DISPLAY_OFFSET_Y)
        .color_order(ColorOrder::Rgb)
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst_pin)
        .init(&mut Delay::new_default())
        .unwrap();

    log::info!("Display initialized");

    // Clear display to black
    display.clear(Rgb565::BLACK).unwrap();

    // Calculate display parameters
    let center_x = DISPLAY_WIDTH / 2;
    let center_y = DISPLAY_HEIGHT / 2;
    let max_diameter = ((DISPLAY_WIDTH * DISPLAY_WIDTH + DISPLAY_HEIGHT * DISPLAY_HEIGHT) as f32).sqrt() as u32;
    let display_rect = Rectangle::new(Point::new(0, 0), Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    log::info!("Display dimensions: {}x{}, max diameter: {}", DISPLAY_WIDTH, DISPLAY_HEIGHT, max_diameter);

    // Draw initial background
    display_rect.into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(&mut display)
        .unwrap();
    log::info!("initial draw");

    let start_time = Instant::now();

    // Animation loop
    loop {
        let elapsed_ms = start_time.elapsed().as_millis() as u32;
        let position_in_cycle = (elapsed_ms % CYCLE_DURATION_MS) as f32 / CYCLE_DURATION_MS as f32;

        // Linear decrease from max to 0 over the cycle
        let current_diameter = (max_diameter as f32 * (1.0 - position_in_cycle)) as u32;

        // Draw background
        display_rect.into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
            .draw(&mut display)
            .unwrap();

        // Draw circle with current diameter
        if current_diameter > 0 {
            let circle = Circle::with_center(
                Point::new(center_x as i32, center_y as i32),
                current_diameter
            );
            circle.into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, CIRCLE_STROKE_WIDTH))
                .draw(&mut display)
                .unwrap();
        }

        thread::sleep(Duration::from_millis(FRAME_DELAY_MS));
    }
}
