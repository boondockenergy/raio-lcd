use crate::{RaioDisplay, RaioRegister};

pub enum S0ColorDepth {
    EightBpp = 0b00,
    SixteenBpp = 0b01,
    TwentyFourBpp = 0b10,
}

pub enum S1ColorDepth {
    EightBpp = 0b000,
    SixteenBpp = 0b001,
    TwentyFourBpp = 0b010,
    ConstantColor = 0b011,
    EightBppAlphaBlend = 0b100,
    SixteenBppAlphaBlend = 0b101,
}

pub enum DestColorDepth {
    EightBpp = 0b00,
    SixteenBpp = 0b01,
    TwentyFourBpp = 0b10,
}

pub enum BteRasterOpCode {
    Black = 0b0000,
    S0MulS1 = 0b1000,
    S1 = 0b1010,
    S0 = 0b1100,
    S0PlusS1 = 0b1110,
    White = 0b1111,
}

pub enum BteOpCode {
    PatternFill = 0b0110,
    SolidFill = 0b1100,
}

impl <'a> RaioDisplay<'a> {

    pub async fn bte_set_dest_addr(&mut self, start_address: u32) {
        self.write_register32(crate::RaioRegister::DtStr0, start_address).await;
    }

    pub async fn bte_set_dest_position(&mut self, x: u16, y: u16) {
        self.write_register16(crate::RaioRegister::DtX0, x).await;
        self.write_register16(crate::RaioRegister::DtY0, y).await;
    }

    pub async fn bte_set_color_depth(&mut self, s0: S0ColorDepth, s1: S1ColorDepth, dest: DestColorDepth) {
        let bte_color = (s0 as u8) << 5 | (s1 as u8) << 2 | dest as u8;
        self.write_register(crate::RaioRegister::BTE_COLR, bte_color).await;
    }

    pub async fn bte_set_dest_width(&mut self, width: u16) {
        //self.write_register(crate::RaioRegister::DtWth0, (width & 0xff) as u8).await;
        //self.write_register(crate::RaioRegister::DtWth1, (width >> 8) as u8).await;
        self.write_register16(crate::RaioRegister::DtWth0, width).await;
    }

    pub async fn bte_set_width(&mut self, width: u16) {
        self.write_register16(RaioRegister::BTE_WTH0, width).await;
    }

    pub async fn bte_set_height(&mut self, height: u16) {
        //self.write_register(crate::RaioRegister::BTE_HIG0, (height & 0xff) as u8).await;
        //self.write_register(crate::RaioRegister::BTE_HIG1, (height >> 8) as u8).await;
        self.write_register16(crate::RaioRegister::BTE_HIG0, height).await;
    }

    pub async fn bte_set_foreground_color(&mut self, red: u8, green: u8, blue: u8) {
        self.write_register(crate::RaioRegister::FGCR, red).await;
        self.write_register(crate::RaioRegister::FGCG, green).await;
        self.write_register(crate::RaioRegister::FGCB, blue).await;
    }

    pub async fn bte_setup(&mut self, rop: BteRasterOpCode, op: BteOpCode) {
        let ctrl1 = (rop as u8) << 4 | (op as u8);
        self.write_register(crate::RaioRegister::BTE_CTRL1, ctrl1).await;
    }

    pub async fn bte_start(&mut self) {
        let ctrl0 = 1<<4;
        self.write_register(crate::RaioRegister::BTE_CTRL0, ctrl0).await;
        let x = self.read_register(crate::RaioRegister::BTE_CTRL0).await;
        self.bte_wait().await;
    }

    /// Wait for the BTE busy bit to clear
    pub async fn bte_wait(&mut self) {
        while self.read_register(crate::RaioRegister::BTE_CTRL0).await & (1<<4) != 0 {}
    }

    pub async fn bte_alpha(&mut self, alpha: u8) {
        self.write_register(crate::RaioRegister::APB_CTRL, alpha).await;
    }

    /// Fill the entire disply with a solid color
    pub async fn fill(&mut self) {
        self.bte_set_dest_addr(0x0).await;

        self.bte_set_dest_position(0, 0).await;

        self.bte_set_color_depth(
            S0ColorDepth::TwentyFourBpp,
            S1ColorDepth::TwentyFourBpp,
            DestColorDepth::TwentyFourBpp
        ).await;

        self.bte_alpha(0x20).await;

        self.bte_set_dest_width(1024).await;
        self.bte_set_width(1024).await;
        self.bte_set_height(600).await;

        //self.bte_set_foreground_color(0xff, 0, 0xff).await;
        self.bte_set_foreground_color(0, 0, 0).await;


        self.bte_setup(BteRasterOpCode::White, BteOpCode::SolidFill).await;

        self.bte_start().await;
    }
}
