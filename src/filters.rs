use std::f32::consts::PI;

fn sinc(x: f32) -> f32
{
    if x.abs() < 1e-4 {
	1.0
    } else {
	x.sin() / x
    }
}

fn hanning(n: f32, m: f32) -> f32
{
    0.5 * (1.0 - (2.0*PI*(n / m + 0.5)).cos())
}

pub fn build_sinc(filter_length: usize, cycle_length: usize) -> Vec<f32>
{
    let mut filters = Vec::new();
    let center = (filter_length - 1) / 2;
    for frac in 0..cycle_length {
	let shift = frac as f32 / cycle_length as f32;
	for s in 0.. filter_length {
	    let n = (s as f32 - center as f32) - shift;
	    let x = PI * n;
	    filters.push(sinc(x) * hanning(n, filter_length as f32));
	}
    }
    filters
}

#[test]
fn build_sinc_test()
{
    let filters = build_sinc(3, 3);
    println!("{:?}", filters);
}
