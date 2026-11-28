use csc_r1p_bringup::fm::{NarrowFm, SAMPLE_RATE};
use esp_hal::{
    Blocking,
    dma::{DmaRxStreamBuf, DmaTxBuf, DmaTxStreamBuf},
    gpio::{Input, InputConfig, InputSignal},
    i2s::master::{DataFormat, I2s, I2sRx, I2sTx, TdmConfig},
    peripherals::{DMA_CH0, GPIO1, GPIO2, GPIO3, GPIO44, I2S0},
    time::{Instant, Rate},
};

const BLOCK_BYTES: usize = 256;

pub struct AudioBus {
    frame_sync: Input<'static>,
    rx: I2sRx<'static, Blocking>,
    tx: I2sTx<'static, Blocking>,
    capture: DmaRxStreamBuf,
    pattern: DmaTxBuf,
    audio: DmaTxStreamBuf,
}

impl AudioBus {
    pub fn new(
        i2s: I2S0<'static>,
        dma: DMA_CH0<'static>,
        bclk: GPIO1<'static>,
        frame_sync: GPIO2<'static>,
        adc_data: GPIO3<'static>,
        dac_data: GPIO44<'static>,
    ) -> Self {
        let driver = I2s::new(
            i2s,
            dma,
            TdmConfig::new_tdm_philips()
                .with_sample_rate(Rate::from_hz(SAMPLE_RATE))
                .with_data_format(DataFormat::Data32Channel32),
        )
        .unwrap();

        // The pinned HAL supplies the DMA and framing setup. Leave its clock output
        // builders disconnected and select external clock inputs for both units.
        let bclk = Input::new(bclk, InputConfig::default());
        let frame_sync = Input::new(frame_sync, InputConfig::default());
        for signal in [InputSignal::I2S0I_BCK, InputSignal::I2S0O_BCK] {
            signal.connect_to(&bclk.peripheral_input());
        }
        for signal in [InputSignal::I2S0I_WS, InputSignal::I2S0O_WS] {
            signal.connect_to(&frame_sync.peripheral_input());
        }
        let regs = I2S0::regs();
        regs.tx_conf().modify(|_, w| {
            w.tx_slave_mod().set_bit();
            w.tx_update().set_bit()
        });
        regs.rx_conf().modify(|_, w| {
            w.rx_slave_mod().set_bit();
            w.rx_update().set_bit()
        });
        let rx = driver.i2s_rx.with_din(adc_data).build();
        let tx = driver.i2s_tx.with_dout(dac_data).build();
        let capture = esp_hal::dma_rx_stream_buffer!(1024, BLOCK_BYTES);
        let mut pattern = esp_hal::dma_tx_buffer!(1024).unwrap();
        // Distinct left/right words make slot order and the I²S one-bit shift
        // visible on a logic analyzer. The board keeps the DAC muted throughout.
        for frame in pattern.as_mut_slice().chunks_exact_mut(8) {
            frame.copy_from_slice(&[0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0xc0]);
        }
        Self {
            frame_sync,
            rx,
            tx,
            capture,
            pattern,
            audio: esp_hal::dma_tx_stream_buffer!(2048, BLOCK_BYTES),
        }
    }

    fn has_frame_sync(&self) -> bool {
        let first_level = self.frame_sync.is_high();
        let start = Instant::now();
        while self.frame_sync.is_high() == first_level {
            if start.elapsed().as_millis() >= 2 {
                return false;
            }
        }
        true
    }

    pub fn probe(mut self) -> Self {
        if !self.has_frame_sync() {
            esp_println::println!("AUDIO BUS: no frame-sync edge");
            return self;
        }
        let mut receive = match self.rx.read(self.capture) {
            Ok(transfer) => transfer,
            Err((error, rx, capture)) => {
                esp_println::println!("AUDIO BUS RX start: {error:?}");
                self.rx = rx;
                self.capture = capture;
                return self;
            }
        };
        let transmit = match self.tx.write(self.pattern) {
            Ok(transfer) => transfer,
            Err((error, tx, pattern)) => {
                esp_println::println!("AUDIO BUS TX start: {error:?}");
                (self.rx, self.capture) = receive.stop();
                self.tx = tx;
                self.pattern = pattern;
                return self;
            }
        };

        let start = Instant::now();
        let mut block = [0; BLOCK_BYTES];
        let mut received = 0;
        while start.elapsed().as_millis() < 20 {
            if received == 0 && receive.available_bytes() >= BLOCK_BYTES {
                received = receive.pop(&mut block);
            }
            if received != 0 && transmit.is_done() {
                break;
            }
        }
        (self.rx, self.capture) = receive.stop();
        if transmit.is_done() {
            let (status, tx, pattern) = transmit.wait();
            (self.tx, self.pattern) = (tx, pattern);
            esp_println::println!("AUDIO BUS TX completion={status:?}");
        } else {
            (self.tx, self.pattern) = transmit.stop();
            esp_println::println!("AUDIO BUS TX timed out");
        }
        if received >= 8 {
            let left = i32::from_le_bytes(block[..4].try_into().unwrap());
            let right = i32::from_le_bytes(block[4..8].try_into().unwrap());
            esp_println::println!(
                "AUDIO BUS RX bytes={received} first slots={left:#010x}/{right:#010x}"
            );
        } else {
            esp_println::println!("AUDIO BUS RX timed out");
        }
        self
    }

    /// Exercise continuous receive and transmit DMA with the DAC kept muted.
    pub fn fm_stream_test(mut self, rf_ready: impl Fn() -> bool) -> Self {
        if !self.has_frame_sync() {
            esp_println::println!("FM STREAM: no frame-sync edge");
            return self;
        }
        // Each run starts with four silent descriptors. Re-create the view
        // so a retried command cannot transmit data left by the previous run.
        let (desc, data) = self.audio.split();
        let mut audio = DmaTxStreamBuf::new(desc, data).unwrap();
        assert_eq!(audio.push(&[0; 4 * BLOCK_BYTES]), 4 * BLOCK_BYTES);

        let mut receive = match self.rx.read(self.capture) {
            Ok(transfer) => transfer,
            Err((error, rx, capture)) => {
                esp_println::println!("FM STREAM RX start: {error:?}");
                self.rx = rx;
                self.capture = capture;
                self.audio = audio;
                return self;
            }
        };
        let mut transmit = match self.tx.write(audio) {
            Ok(transfer) => transfer,
            Err((error, tx, audio)) => {
                esp_println::println!("FM STREAM TX start: {error:?}");
                (self.rx, self.capture) = receive.stop();
                self.tx = tx;
                self.audio = audio;
                return self;
            }
        };

        let start = Instant::now();
        let mut progress = start;
        let mut fm = NarrowFm::default();
        let mut frames = 0;
        let mut peak = 0;
        let mut block = [0; BLOCK_BYTES];
        const DURATION_MS: u64 = 2_000;
        const MIN_FRAMES: usize = SAMPLE_RATE as usize * 2 * 95 / 100;
        let status = loop {
            if !rf_ready() {
                break "PLL lost lock";
            }
            if start.elapsed().as_millis() >= DURATION_MS {
                if frames < MIN_FRAMES {
                    break "frame rate low";
                }
                break "complete";
            }
            if progress.elapsed().as_millis() >= 3 {
                break "DMA stalled or clocks lost";
            }
            if start.elapsed().as_millis() > 1 && transmit.is_done() {
                break "TX underrun";
            }
            if receive.available_bytes() < BLOCK_BYTES || transmit.available_bytes() < BLOCK_BYTES {
                continue;
            }
            if receive.pop(&mut block) != BLOCK_BYTES {
                break "partial RX block";
            }
            for word in block.chunks_exact_mut(8) {
                let i = i32::from_le_bytes(word[..4].try_into().unwrap());
                let q = i32::from_le_bytes(word[4..].try_into().unwrap());
                let audio = fm.process(i, q);
                peak = peak.max(audio.unsigned_abs());
                word[..4].copy_from_slice(&audio.to_le_bytes());
                word[4..].copy_from_slice(&audio.to_le_bytes());
            }
            if transmit.push(&block) != BLOCK_BYTES {
                break "partial TX block";
            }
            frames += BLOCK_BYTES / 8;
            progress = Instant::now();
        };

        (self.tx, self.audio) = transmit.stop();
        (self.rx, self.capture) = receive.stop();
        esp_println::println!("FM STREAM {status}; frames={frames} peak={peak} (DAC muted)");
        self
    }
}
