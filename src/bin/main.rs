#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio, spi::Async, peripherals::{SPI0, PIN_5}};
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::spi::{Config, Spi};

use core::fmt::Write;
use heapless::String;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Startup");

    // reset 7
    let mut lcd_reset = Output::new(p.PIN_7, Level::Low);
    Timer::after_millis(500).await;
    lcd_reset.set_high();

    // backlight 9
    let _lcd_backlight = Output::new(p.PIN_9, Level::High);

    let miso = p.PIN_4;
    let mosi = p.PIN_3;
    let clk = p.PIN_2;

    let cs: Output<'static, PIN_5> = Output::new(p.PIN_5, Level::High);
    let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());
    spi.set_frequency(24_000_000);

    let mut display = raio_lcd::RaioDisplay::new(spi, cs);

    display.init_display().await;
    display.enable_display(true).await;

    display.enable_test_pattern(true).await;
    Timer::after_millis(500).await;
    display.enable_test_pattern(false).await;

    Timer::after_millis(100).await;
    display.set_cursor(0, 0);

    display.fill().await;

    let buf = [0x00;32];
    display.write(&buf).await;

    display.enable_text_mode(true).await;

    let mut count = 0;
    let mut msg: String<128> = String::new();
    loop {

        display.set_text_cursor(0, 0).await;

        core::write!(&mut msg, "Hello Rust Count {}", count).unwrap();

        display.write_text(msg.as_str()).await;
        msg.clear();

        count += 1;

        Timer::after_millis(1).await;
    }

}