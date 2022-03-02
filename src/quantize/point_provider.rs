pub trait PointProvider {
    fn from_int(&self, argb: u32) -> [f64; 3];
    fn to_int(&self, point: &[f64]) -> u32;
    fn distance(&self, a: &[f64], b: &[f64]) -> f64;
}
