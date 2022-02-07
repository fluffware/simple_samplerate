use std::f32::consts::PI;

struct FractionalPosition {
    num: u32,
    denom: u32,
    frac: u32,
    end: bool,
}

impl FractionalPosition {
    pub fn new(num: u32, denom: u32) -> FractionalPosition {
        FractionalPosition {
            num: num % denom,
            denom,
            frac: 0,
            end: false,
        }
    }
}

impl Iterator for FractionalPosition {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        let f = self.frac as f32 / self.denom as f32;
        self.frac += self.num;
        if self.frac >= self.denom {
            self.frac -= self.denom;
        }
        self.end = self.frac == 0;
        Some(f)
    }
}

fn sinc(x: f32) -> f32 {
    if x.abs() < 1e-4 {
        1.0
    } else {
        x.sin() / x
    }
}

fn hanning(n: f32, m: f32) -> f32 {
    0.5 * (1.0 - (2.0 * PI * (n / m + 0.5)).cos())
}

pub fn build_sinc(filter_length: usize, from_rate: u32, to_rate: u32) -> Vec<f32> {
    let mut filters = Vec::new();
    let center = (filter_length - 1) / 2;
    let frac_iter = FractionalPosition::new(from_rate, to_rate);
    for shift in frac_iter {
        for s in 0..filter_length {
            let n = (s as f32 - center as f32) - shift;
            let x = PI * n;
            filters.push(sinc(x) * hanning(n, filter_length as f32));
        }
    }
    filters
}

#[test]
fn build_sinc_test() {
    let filters = build_sinc(3, 3, 4);
    println!("{:?}", filters);
}

#[test]
fn pos_iter_test() {
    let f = FractionalPosition::new(7, 4);
    assert_eq!(f.collect::<Vec<f32>>().as_slice(), &[0.0, 0.75, 0.5, 0.25]);
    let f = FractionalPosition::new(1, 8);
    assert_eq!(
        f.collect::<Vec<f32>>().as_slice(),
        &[0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]
    );
}
