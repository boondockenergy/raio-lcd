#![no_std]
#![no_main]

use defmt::*;
use embassy_rp::{gpio, spi::Async, peripherals::{SPI0, PIN_5}};
use embassy_time::Timer;
use gpio::Output;
use embassy_rp::spi::Spi;

mod bte;

#[derive(Clone)]
enum RaioRegister {
    Srr = 0x00,
    Ccr = 0x01,
    Macr = 0x02,
    Icr = 0x03,
    Mrwdp = 0x04,
    Ppllc1 = 0x05,
    Ppllc2 = 0x06,
    Mpllc1 = 0x07,
    Mpllc2 = 0x08,
    Spllc1 = 0x09,
    Spllc2 = 0x0A,

    Inten = 0x0B,
    Intf = 0x0C,
    Mintfr = 0x0D,
    Puenr = 0x0E,

    Psfsr = 0x0F,
    MPWCTR = 0x10,
    Pipcdep = 0x11,
    DPCR = 0x12,
    PCSR = 0x13,
    HDWR = 0x14,
    HDWFTR = 0x15,
    HNDR = 0x16,
    HNDFTR = 0x17,
    HSTR = 0x18,
    HPWR = 0x19,
    VDHR0 = 0x1A,
    VDHR1 = 0x1B,
    VNDR0 = 0x1C,
    VNDR1 = 0x1D,
    VSTR = 0x1E,
    VPWR = 0x1F,

    MISA0 = 0x20,
    MISA1 = 0x21,
    MISA2 = 0x22,
    MISA3 = 0x23,

    MIW0 = 0x24,
    MIW1 = 0x25,
    MWULX0 = 0x26,
    MWULX1 = 0x27,
    MWULY0 = 0x28,
    MWULY1 = 0x29,

    CvsSa = 0x50,
    CVS_IMWTH0 = 0x54,
    CVS_IMWTH1 = 0x55,

    AW_WTH0 = 0x5A,
    AW_WTH1 = 0x5B,
    AW_HT0 = 0x5C,
    AW_HT1 = 0x5D,

    AW_COLOR = 0x5E,

    CURH0 = 0x5F,
    CURH1 = 0x60,
    CURV0 = 0x61,
    CURV1 = 0x62,

    F_CURX0 = 0x63,
    F_CURX1 = 0x64,

    F_CURY0 = 0x65,
    F_CURY1 = 0x66,

    // Block Transfer Engine Registers
    BTE_CTRL0 = 0x90,
    BTE_CTRL1 = 0x91,
    BTE_COLR = 0x92,
    S0_STR0 = 0x93,
    S0_STR1 = 0x94,
    S0_STR2 = 0x95,
    S0_STR3 = 0x96,
    S0_WTH0 = 0x97,
    S0_WTH1 = 0x98,
    S0_X0 = 0x99,
    S0_X1 = 0x9A,
    S0_Y0 = 0x9B,
    S0_Y1 = 0x9C,
    S1_STR0 = 0x9D,
    S1_STR1 = 0x9E,
    S1_STR2 = 0x9F,
    S1_STR3 = 0xA0,
    S1_WTH0 = 0xA1,
    S1_WTH1 = 0xA2,
    S1_X0 = 0xA3,
    S1_X1 = 0xA4,
    S1_Y0 = 0xA5,
    S1_Y1 = 0xA6,
    DtStr0 = 0xA7,
    DtStr1 = 0xA8,
    DtStr2 = 0xA9,
    DtStr3 = 0xAA,
    DtWth0 = 0xAB,
    DtWth1 = 0xAC,
    DtX0 = 0xAD,
    DtX1 = 0xAE,
    DtY0 = 0xAF,
    DtY1 = 0xB0,
    BTE_WTH0 = 0xB1,
    BTE_WTH1 = 0xB2,
    BTE_HIG0 = 0xB3,
    BTE_HIG1 = 0xB4,

    // Alpha blending
    APB_CTRL = 0xB5,

    // Text foreground color
    FGCR = 0xD2,
    FGCG = 0xD3,
    FGCB = 0xD4,

    // Text background color
    BGCR = 0xD5,
    BGCG = 0xD6,
    BGCB = 0xD7,

    SDRAR = 0xE0,
    SDR_REF_ITVL0 = 0xE2,
    SDR_REF_ITVL1 = 0xE3,
    SDRCR = 0xE4,
}

struct SdramConfig {
    clock: u32,
    cas_latency: u8,
    banks: u8,
    rows: u8,
    cols: u8,
    refresh_time: u32
}

struct DisplayConfig {
    width: u16,
    height: u16,
    pixel_clock_khz: u32,
    horiz_front_porch: u16,
    horiz_back_porch: u16,
    hsync_pulse_width: u8,
    vert_front_porch: u16,
    vert_back_porch: u16,
    vsync_pulse_width: u8
}

pub struct RaioDisplay<'a> {
    spi: Spi<'a, embassy_rp::peripherals::SPI0, Async>,
    cs: Output<'a, PIN_5>,
}

impl <'a> RaioDisplay<'a> {
    pub fn new(spi: Spi<'a, SPI0, Async>, cs: Output<'a, PIN_5>) -> Self {
        Self {
            spi,
            cs,
        }
    }

    pub async fn wait(&mut self) {
        defmt::trace!("POLL");
        while self.read_status().await & (1<<3) != 0 {
            info!("Not ready");
        }
    }

    pub async fn read_status(&mut self) -> u8 {
        let tx_buf = [0b01000000];
        let mut rx_buf = [0_u8; 2];

        self.cs.set_low();
        self.spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        self.cs.set_high();

        defmt::trace!("Status {:b}", rx_buf[1]);

        rx_buf[1]
    }

    pub async fn cmd_write(&mut self, reg: u8) {
        let tx_buf = [0u8, reg];
        self.cs.set_low();
        self.spi.write(&tx_buf).await.unwrap();
        self.cs.set_high();
    }

    pub async fn data_write(&mut self, data: u8) {
        let tx_buf = [0b10000000, data];
        self.cs.set_low();
        self.spi.write(&tx_buf).await.unwrap();
        self.cs.set_high();
    }

    pub async fn data_read(&mut self) -> u8 {
        let tx_buf = [0b11000000];
        let mut rx_buf = [0u8;2];
        self.cs.set_low();
        self.spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        self.cs.set_high();

        rx_buf[1]
    }

    async fn write_register(&mut self, reg: RaioRegister, data: u8) {
        //defmt::trace!("Write {:#?} => {:b}", defmt::Debug2Format(&reg), data);
        self.cmd_write(reg as u8).await;
        self.data_write(data).await;
    }

    async fn write_register16(&mut self, reg: RaioRegister, data: u16) {
        let tx_buf = [0u8, reg.clone() as u8, 0b10000000];

        //defmt::debug!("Write16: {:x}", data);

        /*
        self.cs.set_low();
        self.spi.write(&tx_buf).await.unwrap();
        self.spi.write(&data.to_be_bytes()).await.unwrap();
        self.cs.set_high();
        */

        let reg_num = reg as u8;

        self.cmd_write(reg_num).await;
        self.data_write((data & 0xff) as u8).await;
        self.cmd_write(reg_num + 1).await;
        self.data_write((data >> 8) as u8).await;
    }

    async fn write_register32(&mut self, reg: RaioRegister, data: u32) {
        let tx_buf = [0u8, reg as u8, 0b10000000];
        self.cs.set_low();
        self.spi.write(&tx_buf).await.unwrap();
        self.spi.write(&data.to_be_bytes()).await.unwrap();
        self.cs.set_high();
    }

    async fn read_register(&mut self, reg: RaioRegister) -> u8 {
        self.cmd_write(reg as u8).await;
        self.data_read().await
    }

    pub async fn set_display_size(&mut self, width: u16, height: u16) {
        self.write_register(RaioRegister::HDWR, (width / 8) as u8 - 1).await;
        self.write_register(RaioRegister::HDWFTR, (width % 8) as u8).await;

        self.write_register(RaioRegister::VDHR0, (height - 1 & 0xFF) as u8).await;
        self.write_register(RaioRegister::VDHR1, (height - 1 >> 8) as u8).await;
    }

    pub async fn set_back_porch(&mut self, horiz: u16, vert: u16) {
        self.write_register(RaioRegister::HNDR, (horiz / 8) as u8 - 1).await;
        self.write_register(RaioRegister::HNDFTR, (horiz % 8) as u8).await;

        self.write_register(RaioRegister::VNDR0, (vert - 1 & 0xFF) as u8).await;
        self.write_register(RaioRegister::VNDR1, (vert - 1 >> 8) as u8).await;
    }

    pub async fn set_front_porch(&mut self, horiz: u16, vert: u16) {
        self.write_register(RaioRegister::HSTR, (horiz + 4 / 8) as u8 - 1).await;
        self.write_register(RaioRegister::VSTR, (vert - 1) as u8).await;
    }

    pub async fn init_memory(&mut self) {

        let refresh_interval = 0x20Du16;

        // Set memory type in SDRAR. See section 19.12 reference settings
        // Set to 128Mbit 4 banks, row size 4096, col size 512
        self.write_register(RaioRegister::SDRAR, 0x29).await;

        self.write_register(RaioRegister::SDR_REF_ITVL0, (refresh_interval & 0xff) as u8).await;
        self.write_register(RaioRegister::SDR_REF_ITVL1, (refresh_interval >> 8) as u8).await;

        // Start SDRAM initialization
        let mut sdrcr = self.read_register(RaioRegister::SDRCR).await;
        sdrcr |= 1;
        self.write_register(RaioRegister::SDRCR, sdrcr).await;

        // Wait for SDRAM init to complete
        while self.read_register(RaioRegister::SDRCR).await & 0x1 == 0{
            defmt::debug!("SDRAM not ready");
            Timer::after_millis(200).await;
        }
    }

    pub async fn init_display(&mut self) {
        self.wait().await;

        self.init_memory().await;

        self.set_display_size(1024, 600).await;
        self.set_back_porch(160, 23).await;
        self.set_front_porch(160, 12).await;

        // Set main image width
        self.write_register(RaioRegister::MIW0, (1024 & 0xff) as u8).await;
        self.write_register(RaioRegister::MIW1, (1024 >> 8) as u8).await;

        self.write_register(RaioRegister::AW_WTH0, (1024 & 0xff) as u8).await;
        self.write_register(RaioRegister::AW_WTH1, (1024 >> 8) as u8).await;
        self.write_register(RaioRegister::AW_HT0, (600 & 0xff) as u8).await;
        self.write_register(RaioRegister::AW_HT1, (600 >> 8) as u8).await;

        self.write_register(RaioRegister::CVS_IMWTH0, (1024 & 0xff) as u8).await;
        self.write_register(RaioRegister::CVS_IMWTH1, (1024 >> 8) as u8).await;

        // Set Main window control register to 16bpp
        //self.write_register(RaioRegister::MPWCTR, 0x4).await;
        // 24bpp
        self.write_register(RaioRegister::MPWCTR, 0x8).await;

        // 24bpp
        self.write_register(RaioRegister::AW_COLOR, 0x2).await;

        self.write_register(RaioRegister::Ccr, 0x00).await;

        // HSYNC active high, VSYNC active high, XDE polarity high
        self.write_register(RaioRegister::PCSR, (1<<6) | (1<<7)).await;

        // PCLK inverted, Display on, color bar enable, BRG 
        self.write_register(RaioRegister::DPCR, (1<<7) | (1<<6) | 0b000).await;
    }

    /// Turn the display on
    pub async fn enable_display(&mut self, enabled: bool) {
        let mut dpcr = self.read_register(RaioRegister::DPCR).await;
        match enabled {
            true => dpcr |= (1<<6),
            false => dpcr &= !(1<<6),
        }
        self.write_register(RaioRegister::DPCR, dpcr).await;
    }

    pub async fn enable_test_pattern(&mut self, enabled: bool) {
        let mut dpcr = self.read_register(RaioRegister::DPCR).await;

        match enabled {
            true => dpcr |= 1<<5,
            false => dpcr &= !(1<<5),
        }

        self.write_register(RaioRegister::DPCR, dpcr).await;
    }

    pub async fn enable_text_mode(&mut self, enabled: bool) {
        let mut icr = self.read_register(RaioRegister::Icr).await;
        match enabled {
            true => icr |= (1<<2),
            false => icr &= !(1<<2),
        }
        self.write_register(RaioRegister::Icr, icr).await;

        self.write_register(RaioRegister::FGCR, 0xff).await;
    }

    pub async fn set_cursor(&mut self, x: u16, y: u16) {
        self.write_register(RaioRegister::CURH0, (x & 0xff) as u8).await;
        self.write_register(RaioRegister::CURH1, (x >> 8) as u8).await;
    }

    pub async fn set_text_cursor(&mut self, x: u16, y: u16) {
        self.write_register(RaioRegister::F_CURX0, (x & 0xff) as u8).await;
        self.write_register(RaioRegister::F_CURX1, (x >> 8) as u8).await;

        self.write_register(RaioRegister::F_CURY0, (y & 0xff) as u8).await;
        self.write_register(RaioRegister::F_CURY1, (y >> 8) as u8).await;
    }

    pub async fn get_text_cursor_y(&mut self) -> u16 {
        let mut y = self.read_register(RaioRegister::F_CURY0).await as u16;
        y |= (self.read_register(RaioRegister::F_CURY1).await as u16) << 8;
        y
    }

    pub async fn write(&mut self, data: &[u8]) {
        let tx_buf = [0u8, RaioRegister::Mrwdp as u8, 0b10000000];
        self.cs.set_low();
        self.spi.write(&tx_buf).await.unwrap();
        self.spi.write(data).await.unwrap();
        self.cs.set_high();
    }

    pub async fn write_text(&mut self, data: &str) {

        let chunks = data.as_bytes().chunks(16);

        for chunk in chunks {
            // Wait for FIFO empty, and write up to 64 bytes at a time
            while self.read_status().await & (1<<6) == 0 {};

            self.write(chunk).await;

            self.wait().await;
        }
    }

}


