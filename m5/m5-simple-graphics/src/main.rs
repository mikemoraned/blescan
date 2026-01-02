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
    // M5StickC Plus2 requires both RGB order and color inversion
    let mut display = Builder::new(mipidsi::models::ST7789, di)
        .display_size(135, 240)
        .display_offset(52, 40)  // M5StickC Plus2 specific offsets
        .color_order(ColorOrder::Rgb)  // RGB order (not BGR!)
        .invert_colors(ColorInversion::Inverted)  // Colors must be inverted
        .reset_pin(rst_pin)
        .init(&mut Delay::new_default())
        .unwrap();

    log::info!("Display initialized");

    // Clear display to black
    display.clear(Rgb565::BLACK).unwrap();

    // Draw red background rectangle
    let red_rect = Rectangle::new(Point::new(0, 0), Size::new(135, 240));
    red_rect.into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut display)
        .unwrap();
    log::info!("Red background drawn");

    // Get display dimensions (portrait: 135 wide x 240 tall)
    let width = 135;
    let height = 240;
    let center_x = width / 2;
    let center_y = height / 2;

    // Calculate max diameter as the diagonal of the display
    let max_diameter = ((width * width + height * height) as f32).sqrt() as u32;

    log::info!("Display dimensions: {}x{}, max diameter: {}", width, height, max_diameter);

    // Animation parameters
    let cycle_duration_ms = 2500;
    let frame_delay_ms = 8;
    let start_time = Instant::now();

    // Animation loop
    loop {
        let elapsed_ms = start_time.elapsed().as_millis() as u32;
        let position_in_cycle = (elapsed_ms % cycle_duration_ms) as f32 / cycle_duration_ms as f32;

        // Linear decrease from max to 0 over the cycle
        // t=0: diameter=max, t=0.5: diameter=max/2, t→1: diameter→0, t=1: jump to max
        let current_diameter = (max_diameter as f32 * (1.0 - position_in_cycle)) as u32;

        // Redraw red background
        let red_rect = Rectangle::new(Point::new(0, 0), Size::new(135, 240));
        red_rect.into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
            .draw(&mut display)
            .unwrap();

        // Draw white circle with current diameter
        if current_diameter > 0 {
            let circle = Circle::with_center(
                Point::new(center_x as i32, center_y as i32),
                current_diameter
            );
            circle.into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 2))
                .draw(&mut display)
                .unwrap();
        }

        thread::sleep(Duration::from_millis(frame_delay_ms as u64));
    }
}
