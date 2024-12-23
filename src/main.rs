use rodio::cpal::SampleRate;
use rodio::source::{Amplify, Function, SignalGenerator, Source, TakeDuration};
use rodio::{OutputStream, Sink};
use std::time::Duration;

const AMPL: f32 = 0.2;
const KHZ_1: f32 = 1000.0;
const KHZ_2: f32 = 2000.0;
const DURATION_SCALE: u64 = 1;

fn get_sg(freq: f32, duration_ms: u64) -> Amplify<TakeDuration<SignalGenerator>> {
    SignalGenerator::new(SampleRate(64000), freq, Function::Square)
        .take_duration(Duration::from_millis(duration_ms * DURATION_SCALE))
        .amplify(AMPL)
}

fn lead_sync(sink: &Sink) {
    sink.append(get_sg(KHZ_1, 4000));
}

fn mid_sync(sink: &Sink) {
    sink.append(get_sg(KHZ_2, 2000));
}

fn fail_sync(sink: &Sink) {
    sink.append(get_sg(KHZ_2, 2000));
}

fn bit_0(sink: &Sink) {
    sink.append(get_sg(KHZ_2, 6));
    sink.append(get_sg(KHZ_1, 3));
}

fn bit_1(sink: &Sink) {
    sink.append(get_sg(KHZ_2, 3));
    sink.append(get_sg(KHZ_1, 6));
}

fn byte(sink: &Sink, byte: u8) {
    // start with a zero bit
    bit_0(sink);
    // iterate the bits of byte
    for i in (0..8).rev() {
        if (byte & (1 << i)) != 0 {
            bit_1(sink);
        } else {
            bit_0(sink);
        }
    }
    // end with a one bit
    bit_1(sink);
}

fn main() {
    // _stream must live as long as the sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // LEAD SYNC
    lead_sync(&sink);
    // Filename
    byte(&sink, 0xAA);
    byte(&sink, 0xFF);
    // Start address
    byte(&sink, 0x00);
    byte(&sink, 0x20);
    // End address
    byte(&sink, 0x03);
    byte(&sink, 0x20);
    // Check sum
    byte(&sink, 0x06);
    // MID SYNC
    mid_sync(&sink);
    // Data
    byte(&sink, 0x00);
    byte(&sink, 0x01);
    byte(&sink, 0x02);
    byte(&sink, 0x03);
    // FAIL SYNC
    fail_sync(&sink);

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();
}
