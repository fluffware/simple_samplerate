use crate::error::Error;
use crate::filtering::FilteringContext;
use crate::filters;
use crate::sample::Sample;

/// Samplerate conversion state
///
/// Type I is the type of the input samples.  Resampling is done using
/// a sequence of precalculated filters. The number of filters depends
/// on the rates but is never more than the output rate. E.g. doubling
/// the samplerate needs two filters and conversion from 44100 to
/// 48000 needs 160 filters.
pub struct Samplerate<I> {
    filtering: FilteringContext,
    overlap: Vec<I>,
    filter_offset: usize,
}
use num::integer::gcd;

impl<I> Samplerate<I>
where
    I: Sample + Copy,
{
    /// Create a new samplerate converter using a sinc filter of length 8
    ///
    /// There is no low pass filtering so there may be aliasing when
    /// downsampling.
    pub fn new(from_rate: u32, to_rate: u32, channels: usize) -> Result<Samplerate<I>, Error> {
        let filter_count = (to_rate / gcd(from_rate, to_rate)) as usize;
        let filters = filters::build_sinc(8, filter_count);
        Ok(Self::from_filters(
            filters, 8, 3, from_rate, to_rate, channels,
        ))
    }

    /// Create a new samplerate converter that uses the supplied
    /// filters for resampling.
    pub fn from_filters(
        filters: Vec<f32>,
        filter_length: usize,
        filter_offset: usize,
        from_rate: u32,
        to_rate: u32,
        channels: usize,
    ) -> Samplerate<I> {
        let overlap_len = (filter_length - 1) * channels * 2;
        let mut overlap = Vec::with_capacity(overlap_len);
        overlap.resize(filter_offset * channels, I::OFFSET);
        Samplerate {
            filtering: FilteringContext::new(
                filters,
                filter_length,
                from_rate as usize,
                to_rate as usize,
                channels,
            ),
            overlap,
            filter_offset,
        }
    }

    /// Process a block of samples.
    ///
    /// Channels are interleaved. The length of the ouput buffer
    /// should be at least the length of the input buffer scaled by
    /// the convertion ratio plus the filter length.
    pub fn process_buffer<O>(&mut self, input: &[I], output: &mut [O]) -> usize
    where
        O: Sample,
    {
        if input.is_empty() {
            return 0;
        }
        let channels = self.filtering.channels;
        let min_input_length = self.filtering.filter_length * channels;
        let overlap_length = (self.filtering.filter_length - 1) * channels;
        let copy_length = input.len().min(overlap_length);
        self.overlap.extend_from_slice(&input[..copy_length]);
        let mut output_index = 0;
        if self.overlap.len() >= min_input_length {
            output_index += self.filtering.apply_filters(&self.overlap, output);
        }
        if input.len() < min_input_length {
            self.overlap.drain(0..self.overlap.len() - overlap_length);
        } else {
            output_index += self
                .filtering
                .apply_filters(input, &mut output[output_index..]);
            self.overlap.clear();
            self.overlap
                .extend_from_slice(&input[input.len() - overlap_length..]);
        }
	output_index
    }
    /// Process the last block of samples.
    ///
    /// Same as `process_buffer` but adds silence to process the
    /// last samples
    pub fn process_last_buffer<O>(&mut self, input: &[I], output: &mut [O]) -> usize
    where
        O: Sample,
    {
        let mut output_index = self.process_buffer(input, output);
        self.overlap.resize(
            self.overlap.len()
                + (self.filtering.filter_length - self.filter_offset - 1) * self.filtering.channels,
            I::OFFSET,
        );
        output_index += self
            .filtering
            .apply_filters(&self.overlap, &mut output[output_index..]);
        self.overlap
            .resize(self.filter_offset * self.filtering.channels, I::OFFSET);
        output_index
    }
}

#[cfg(test)]
fn build_test_input(frames: usize, channels: usize) -> Vec<f32> {
    let mut samples: Vec<f32> = Vec::new();
    for f in 0..frames {
        for c in 0..channels {
            samples.push(f as f32 + 0.125 * c as f32);
        }
    }
    samples
}

#[test]
fn test_blocking() {
    let unit_filter = vec![1.0f32];
    let input = build_test_input(20, 2);
    let mut filter = Samplerate::from_filters(unit_filter, 1, 0, 4, 4, 2);
    let mut output = [-1.0; 40];
    let mut out_pos = 0;
    out_pos += filter.process_buffer(&input[0..6], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[6..14], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[14..], &mut output[out_pos..]);
    assert_eq!(&output[..out_pos], input.as_slice());
}

#[test]
fn test_blocking2() {
    let avg_filter = vec![0.5f32, 0.5];
    let input = build_test_input(20, 2);
    let mut filter = Samplerate::from_filters(avg_filter, 2, 0, 4, 4, 2);
    let mut output = [-1.0; 38];
    let mut out_pos = 0;
    out_pos += filter.process_buffer(&input[0..6], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[6..14], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[14..], &mut output[out_pos..]);
    let mut calculated = Vec::new();
    for f in 0..input.len() / 2 - 1 {
        for c in 0..2 {
            calculated.push((input[f * 2 + c] + input[(f + 1) * 2 + c]) / 2.0);
        }
    }
    assert_eq!(&output[..out_pos], calculated.as_slice());
}

#[test]
fn test_blocking3() {
    let unit_filter = vec![0.0, 1.0, 0.0];
    let input = build_test_input(20, 2);
    let mut filter = Samplerate::from_filters(unit_filter, 3, 0, 4, 4, 2);
    let mut output = [-1.0; 36];
    let mut out_pos = 0;
    out_pos += filter.process_buffer(&input[0..6], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[6..12], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[12..14], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[14..], &mut output[out_pos..]);
    assert_eq!(&output[..out_pos], &input[2..38]);
}

#[test]
fn test_blocking4() {
    let unit_filter = vec![0.25, 0.5, 0.25];
    let input = build_test_input(20, 2);
    let mut filter = Samplerate::from_filters(unit_filter, 3, 0, 4, 4, 2);
    let mut output = [-1.0; 36];
    let mut out_pos = 0;
    out_pos += filter.process_buffer(&input[0..6], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[6..12], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[12..14], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[14..], &mut output[out_pos..]);
    assert_eq!(&output[..out_pos], &input[2..38]);
}

#[test]
fn test_filter_offset() {
    let unit_filter = vec![0.0, 1.0, 0.0];
    let input = build_test_input(20, 2);
    let mut filter = Samplerate::from_filters(unit_filter, 3, 1, 4, 4, 2);
    let mut output = [-1.0; 40];
    let mut out_pos = 0;
    out_pos += filter.process_buffer(&input[0..6], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[6..12], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[12..14], &mut output[out_pos..]);
    out_pos += filter.process_last_buffer(&input[14..], &mut output[out_pos..]);
    assert_eq!(&output[..out_pos], input.as_slice());
}

#[test]
fn test_filter_sequence() {
    let unit_filter = vec![0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0];
    let input = build_test_input(20, 2);
    let mut filter = Samplerate::from_filters(unit_filter, 3, 0, 4, 6, 2);
    let mut output = [-1.0; 60];
    let mut out_pos = 0;
    out_pos += filter.process_buffer(&input[0..6], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[6..12], &mut output[out_pos..]);
    out_pos += filter.process_buffer(&input[12..14], &mut output[out_pos..]);
    out_pos += filter.process_last_buffer(&input[14..], &mut output[out_pos..]);
    let mut calculated = Vec::new();
    for f in (1..input.len() / 2).step_by(2) {
        for _ in 0..3 {
            for c in 0..2 {
                println!("{} {}", f, c);
                calculated.push(input[f * 2 + c]);
            }
        }
    }
    assert_eq!(&output[..out_pos], calculated.as_slice());
}
