use simple_samplerate::samplerate::Samplerate;
use std::env;
use std::path::Path;

const BLOCK_SIZE: usize = 1024;

fn convert_rate(
    from_file: &Path,
    to_file: &Path,
    to_rate: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = hound::WavReader::open(&from_file)?;
    let mut spec = reader.spec();
    let mut conv = Samplerate::new(
        spec.sample_rate,
        to_rate,
        spec.channels as usize,
    )
    .unwrap();
    let mut out_buffer: Vec<i16> = Vec::new();
    out_buffer.resize(
        BLOCK_SIZE * to_rate as usize / spec.sample_rate as usize +10,
        0,
    );

    spec.sample_rate = to_rate;
    let mut writer = hound::WavWriter::create(&to_file, spec)?;
    let mut samples;
    samples = Vec::<i16>::new();
    for s in reader.samples::<i16>() {
        match s {
            Ok(s) => {
                samples.push(s);
                if samples.len() >= BLOCK_SIZE {
                    //println!("Writing block");
                    let count = conv.process_buffer(&samples, &mut out_buffer);
                    for s in &out_buffer[..count] {
                        writer.write_sample(*s)?;
                    }
                    samples.clear();
                }
            }
            Err(err) => {
                return Err(format!(
                    "Failed to read samples from file \"{}\": {}",
                    from_file.to_string_lossy(),
                    err
                )
                .into())
            }
        }
    }
    let count = conv.process_last_buffer(&samples, &mut out_buffer);
    for s in &out_buffer[..count] {
        writer.write_sample(*s as i16)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let from_file = args.next().unwrap();
    let to_file = args.next().unwrap();
    let to_rate = args.next().and_then(|s| s.parse().ok()).unwrap_or(44100);
    convert_rate(Path::new(&from_file), Path::new(&to_file), to_rate).unwrap();
}
