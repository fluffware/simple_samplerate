
/// Trait for numeric type that can be used for sample values
pub trait Sample
{
    /// Sample value that corresponds to signal center
    const OFFSET: Self;
    /// Scale the sample to the range [-1.0,1.0]
    fn normailze(self) -> f32;
    /// Scale the range [-1.0,1.0] to the full range of the type
    fn full(s: f32) -> Self; 
}

impl Sample for i16
{
    const OFFSET: Self = 0;
    fn full(s :f32) -> i16
    {
	(32767.0 * s).round() as i16
    }
    fn normailze(self) -> f32
    {
	f32::from(self) * (1.0 / 32767.0)
    }
    
}

impl Sample for u16
{
    const OFFSET: Self = 32768;
    fn full(s :f32) -> u16
    {
	(32767.0 * s + 32768.0).round() as u16 
    }
    
    fn normailze(self) -> f32
    {
	(f32::from(self) - 32768.0) * (1.0 / 32767.0)
    }
    
}

impl Sample for f32 {
    const OFFSET: Self = 0.0;
    fn full(s: f32) -> f32
    {
	s
    }
    fn normailze(self) -> f32
    {
	self
    }
}

#[test]
fn sample_test()
{
    assert_eq!(32767i16.normailze(), 1.0);
    assert_eq!(-32767i16.normailze(), -1.0);
    assert_eq!(0i16.normailze(), 0.0);
    assert_eq!(i16::full(1.0), 32767);
    assert_eq!(i16::full(-1.0), -32767);
    assert_eq!(i16::full(0.0), 0);
    
    assert_eq!(65535u16.normailze(), 1.0);
    assert_eq!(1u16.normailze(), -1.0);
    assert_eq!(32768u16.normailze(), 0.0);
    assert_eq!(u16::full(1.0), 65535u16);
    assert_eq!(u16::full(-1.0), 1);
    assert_eq!(u16::full(0.0), 32768u16);
    
    assert_eq!(0.1.normailze(), 0.1);
    assert_eq!(f32::full(0.1), 0.1);
    
}
