use serde::Serialize;

/// 2D affine transformation matrix stored as [a, b, c, d, e, f].
///
/// Represents the matrix:
/// ```text
/// [a  b  0]
/// [c  d  0]
/// [e  f  1]
/// ```
///
/// Transforms point (x, y) to (a*x + c*y + e, b*x + d*y + f).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Matrix {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Matrix {
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// Identity matrix.
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    /// Translation matrix.
    pub fn translate(tx: f64, ty: f64) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    /// Scaling matrix.
    pub fn scale(sx: f64, sy: f64) -> Self {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    /// Multiply this matrix by another: self * other.
    /// This is the standard PDF matrix concatenation.
    pub fn multiply(&self, other: &Matrix) -> Matrix {
        Matrix::new(
            self.a * other.a + self.b * other.c,
            self.a * other.b + self.b * other.d,
            self.c * other.a + self.d * other.c,
            self.c * other.b + self.d * other.d,
            self.e * other.a + self.f * other.c + other.e,
            self.e * other.b + self.f * other.d + other.f,
        )
    }

    /// Transform a point (x, y) by this matrix.
    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// Get the effective font size from a text rendering matrix.
    /// This is sqrt(b² + d²) which gives the vertical scaling factor.
    pub fn font_size(&self) -> f64 {
        (self.b * self.b + self.d * self.d).sqrt()
    }

    /// Check if this matrix represents an upright orientation
    /// (no rotation or only 180° rotation).
    pub fn is_upright(&self) -> bool {
        self.b.abs() < 1e-6 && self.c.abs() < 1e-6
    }

    /// Return as array [a, b, c, d, e, f].
    pub fn as_array(&self) -> [f64; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<[f64; 6]> for Matrix {
    fn from(arr: [f64; 6]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Matrix::identity();
        let (x, y) = m.transform_point(10.0, 20.0);
        assert!((x - 10.0).abs() < 1e-10);
        assert!((y - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_translate() {
        let m = Matrix::translate(5.0, 10.0);
        let (x, y) = m.transform_point(1.0, 2.0);
        assert!((x - 6.0).abs() < 1e-10);
        assert!((y - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_multiply() {
        // In PDF convention: self.multiply(&other) applies self first, then other.
        // scale first, then translate: scale(2,3).multiply(translate(10,20))
        // point (1,1) → scale → (2,3) → translate → (12,23)
        let a = Matrix::scale(2.0, 3.0);
        let b = Matrix::translate(10.0, 20.0);
        let c = a.multiply(&b);
        let (x, y) = c.transform_point(1.0, 1.0);
        assert!((x - 12.0).abs() < 1e-10);
        assert!((y - 23.0).abs() < 1e-10);
    }
}
