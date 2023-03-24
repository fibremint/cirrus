use audio::{InterleavedBufMut, buf::Interleaved};
use opus::Decoder;

pub struct PacketDecoder {
    decoder: Decoder,
    buf: audio::buf::Interleaved<f32>,
}
impl PacketDecoder {
    pub fn new() -> Result<Self, anyhow::Error> {

        Ok(Self {
            decoder: Decoder::new(48_000, opus::Channels::Stereo)?,
            buf: audio::buf::Interleaved::<f32>::with_topology(2, 960),
        })
    }

    pub fn decode(
        &mut self,
        input_packets: &Vec<u8>,
    ) -> Result<&Interleaved<f32>, anyhow::Error> {

        self.decoder.decode_float(
            &input_packets,
            &mut self.buf.as_interleaved_mut(),
            false
        )?;

        Ok(&self.buf)
    }
}

