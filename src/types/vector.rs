use std::{
    f64::{self, consts::PI},
    fmt,
    ops::{Add, Div, Mul, Sub},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::utils::round_precision;

// Used to deserialize a struct as a tuple.
macro_rules! as_serde_tuple {
    ($(#[$smeta:meta])*
        $svis:vis struct $sname:ident {
            $($fvis:vis $fname:ident : $ftype:ty,)*
    }) => {
        $(#[$smeta])*
        $svis struct $sname {
            $($fvis $fname : $ftype,)*
        }

        impl<'de> Deserialize<'de> for $sname {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>
            {
                #[derive(Deserialize, Serialize)]
                pub struct Array($(pub $ftype,)*);

                Deserialize::deserialize(deserializer)
                    .map(|Array($($fname,)*)| Self { $($fname,)* })
            }
        }

        impl Serialize for $sname {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #[derive(Deserialize, Serialize)]
                pub struct Array($(pub $ftype,)*);

                (Array($(self.$fname.clone(),)*)).serialize(serializer)
            }
        }
    }
}

as_serde_tuple! {
    #[allow(missing_docs)]
    /// Represents a 2D point in space.
    #[derive(Default, Debug, PartialEq, Clone, Copy)]
    pub struct Vector2 {
        pub x: f64,
        pub y: f64,
    }
}

impl Vector2 {
    /// Create a 2D point struct from x and y coordinates.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Create a 2D point struct using `f64::MIN` for x and y coordinates.
    #[must_use]
    pub fn min() -> Self {
        Self {
            x: f64::MIN,
            y: f64::MIN,
        }
    }

    /// Create a 2D point struct using `f64::MAX` for x and y coordinates.
    #[must_use]
    pub fn max() -> Self {
        Self {
            x: f64::MAX,
            y: f64::MAX,
        }
    }

    /// Calculate the distance to another `Vector2` struct.
    #[must_use]
    pub fn distance_to(&self, to: Self) -> f64 {
        ((self.x - to.x) * (self.x - to.x) + (self.y - to.y) * (self.y - to.y)).sqrt()
    }

    /// Computes the angle in radians with respect to the positive x-axis.
    #[must_use]
    pub fn angle(&self) -> f64 {
        (-self.x).atan2(-self.y) + PI
    }

    /// Computes the angle in degrees with respect to the positive x-axis.
    #[must_use]
    pub fn angle_degrees(&self) -> f64 {
        self.angle().to_degrees()
    }

    /// Returns a new `Vector2` incrementing the x coordinate by the given value.
    #[must_use]
    pub fn add_x(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.x += value;
        vector
    }

    /// Returns a new `Vector2` incrementing the y coordinate by the given value.
    #[must_use]
    pub fn add_y(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.y += value;
        vector
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{{x: {}, y: {}}}",
            round_precision(self.x),
            round_precision(self.y)
        )
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec2> for Vector2 {
    fn from(value: glam::Vec2) -> Self {
        Self { x: value.x as f64, y: value.y as f64 }
    }
}

#[cfg(feature = "glam")]
impl From<Vector2> for glam::Vec2 {
    fn from(value: Vector2) -> Self {
        Self { x: value.x as f32, y: value.y as f32 }
    }
}

#[cfg(feature = "glam")]
impl From<glam::DVec2> for Vector2 {
    fn from(value: glam::DVec2) -> Self {
        Self { x: value.x, y: value.y }
    }
}

#[cfg(feature = "glam")]
impl From<Vector2> for glam::DVec2 {
    fn from(value: Vector2) -> Self {
        Self { x: value.x, y: value.y }
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Point2<f32>> for Vector2 {
    fn from(value: nalgebra::Point2<f32>) -> Self {
        Self { x: value.x as f64, y: value.y as f64 }
    }
}

#[cfg(feature = "nalgebra")]
impl From<Vector2> for nalgebra::Point2<f32> {
    fn from(value: Vector2) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Point2<f64>> for Vector2 {
    fn from(value: nalgebra::Point2<f64>) -> Self {
        Self { x: value.x, y: value.y }
    }
}

#[cfg(feature = "nalgebra")]
impl From<Vector2> for nalgebra::Point2<f64> {
    fn from(value: Vector2) -> Self {
        Self::new(value.x, value.y)
    }
}

as_serde_tuple! {
    #[allow(missing_docs)]
    /// Represents a 3D point in space.
    #[derive(Default, Debug, PartialEq, Clone, Copy)]
    pub struct Vector3 {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
}

impl Vector3 {
    /// Create a 3D point struct from x, y and z coordinates.
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create a 3D point struct using `f64::MIN` for x, y and z coordinates.
    #[must_use]
    pub fn min() -> Self {
        Self {
            x: f64::MIN,
            y: f64::MIN,
            z: f64::MIN,
        }
    }

    /// Create a 3D point struct using `f64::MAX` for x, y and z coordinates.
    #[must_use]
    pub fn max() -> Self {
        Self {
            x: f64::MAX,
            y: f64::MAX,
            z: f64::MAX,
        }
    }

    /// Calculate the distance to another `Vector3` struct.
    #[must_use]
    pub fn distance_to(&self, to: Self) -> f64 {
        ((self.x - to.x) * (self.x - to.x)
            + (self.y - to.y) * (self.y - to.y)
            + (self.z - to.z) * (self.z - to.z))
            .sqrt()
    }

    /// Returns a `Vector2` struct using the x and y coordinates from this `Vector3` struct.
    #[must_use]
    pub fn xy(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    /// Returns a `Vector2` struct using the x and z coordinates from this `Vector3` struct.
    #[must_use]
    pub fn xz(&self) -> Vector2 {
        Vector2::new(self.x, self.z)
    }

    /// Returns a `Vector2` struct using the y and z coordinates from this `Vector3` struct.
    #[must_use]
    pub fn yz(&self) -> Vector2 {
        Vector2::new(self.y, self.z)
    }

    /// Returns a new `Vector3` incrementing the x coordinate by the given value.
    #[must_use]
    pub fn add_x(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.x += value;
        vector
    }

    /// Returns a new `Vector3` incrementing the y coordinate by the given value.
    #[must_use]
    pub fn add_y(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.y += value;
        vector
    }

    /// Returns a new `Vector3` incrementing the z coordinate by the given value.
    #[must_use]
    pub fn add_z(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.z += value;
        vector
    }
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Div for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{{x: {}, y: {}, z: {}}}",
            round_precision(self.x),
            round_precision(self.y),
            round_precision(self.z)
        )
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3> for Vector3 {
    fn from(value: glam::Vec3) -> Self {
        Self { x: value.x as f64, y: value.y as f64, z: value.z as f64 }
    }
}

#[cfg(feature = "glam")]
impl From<Vector3> for glam::Vec3 {
    fn from(value: Vector3) -> Self {
        Self { x: value.x as f32, y: value.y as f32, z: value.z as f32 }
    }
}

#[cfg(feature = "glam")]
impl From<glam::DVec3> for Vector3 {
    fn from(value: glam::DVec3) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}

#[cfg(feature = "glam")]
impl From<Vector3> for glam::DVec3 {
    fn from(value: Vector3) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Point3<f32>> for Vector3 {
    fn from(value: nalgebra::Point3<f32>) -> Self {
        Self { x: value.x as f64, y: value.y as f64, z: value.z as f64 }
    }
}

#[cfg(feature = "nalgebra")]
impl From<Vector3> for nalgebra::Point3<f32> {
    fn from(value: Vector3) -> Self {
        Self::new(value.x as f32, value.y as f32, value.z as f32)
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Point3<f64>> for Vector3 {
    fn from(value: nalgebra::Point3<f64>) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}

#[cfg(feature = "nalgebra")]
impl From<Vector3> for nalgebra::Point3<f64> {
    fn from(value: Vector3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_min() {
        let vector = Vector3::min();
        assert!(vector.x == f64::MIN);
        assert!(vector.y == f64::MIN);
        assert!(vector.z == f64::MIN);
    }

    #[test]
    fn test_vector3_max() {
        let vector = Vector3::max();
        assert!(vector.x == f64::MAX);
        assert!(vector.y == f64::MAX);
        assert!(vector.z == f64::MAX);
    }

    #[test]
    fn test_vector3_xy_xz_yz() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        assert!(vector.xy() == Vector2::new(1.0, 2.0));
        assert!(vector.xz() == Vector2::new(1.0, 3.0));
        assert!(vector.yz() == Vector2::new(2.0, 3.0));
    }

    #[test]
    fn test_vector3_add_xyz() {
        let vector = Vector3::default();

        let vector = vector.add_x(1.0);
        assert!(vector.x == 1.0);
        assert!(vector.y == 0.0);
        assert!(vector.z == 0.0);

        let vector = vector.add_y(-1.0);
        assert!(vector.x == 1.0);
        assert!(vector.y == -1.0);
        assert!(vector.z == 0.0);

        let vector = vector.add_z(3.0);
        assert!(vector.x == 1.0);
        assert!(vector.y == -1.0);
        assert!(vector.z == 3.0);
    }

    #[test]
    fn test_vector2_min() {
        let vector = Vector2::min();
        assert!(vector.x == f64::MIN);
        assert!(vector.y == f64::MIN);
    }

    #[test]
    fn test_vector2_max() {
        let vector = Vector2::max();
        assert!(vector.x == f64::MAX);
        assert!(vector.y == f64::MAX);
    }

    #[test]
    fn test_vector2_add_xyz() {
        let vector = Vector2::default();

        let vector = vector.add_x(1.0);
        assert!(vector.x == 1.0);
        assert!(vector.y == 0.0);

        let vector = vector.add_y(-1.0);
        assert!(vector.x == 1.0);
        assert!(vector.y == -1.0);
    }

    #[test]
    fn test_vector2_distance_to() {
        let vector_a = Vector2::new(20.0, 40.0);
        let vector_b = Vector2::new(20.0, 20.0);
        assert!(vector_a.distance_to(vector_b) == 20.0);
    }

    #[test]
    fn test_vector2_angle() {
        let vector = Vector2::new(20.0, 0.0);
        assert!(vector.angle() == f64::consts::PI / 2.0);
    }

    #[test]
    fn test_vector2_angle_degree() {
        let vector = Vector2::new(20.0, 20.0);
        assert!(vector.angle_degrees() == 45.0);
    }

    #[cfg(feature = "glam")]
    #[test]
    fn test_glam_from_into() {
        let v = Vector3::new(23.1, 5.0, 0.0);

        let b: glam::DVec3 = v.into();
        assert_eq!(b, glam::DVec3::new(23.1, 5.0, 0.0));

        let c = Vector3::from(b);
        assert_eq!(c, v);

        let d: glam::Vec3 = c.into();
        assert_eq!(d, glam::Vec3::new(23.1, 5.0, 0.0));

        let v = Vector2::new(23.1, 5.0);

        let b: glam::DVec2 = v.into();
        assert_eq!(b, glam::DVec2::new(23.1, 5.0));

        let c = Vector2::from(b);
        assert_eq!(c, v);

        let d: glam::Vec2 = c.into();
        assert_eq!(d, glam::Vec2::new(23.1, 5.0));
    }

    #[cfg(feature = "nalgebra")]
    #[test]
    fn test_nalgebra_from_into() {
        let v = Vector3::new(23.1, 5.0, 0.0);

        let b: nalgebra::Point3<f64> = v.into();
        assert_eq!(b, nalgebra::Point3::new(23.1, 5.0, 0.0));

        let c = Vector3::from(b);
        assert_eq!(c, v);

        let d: nalgebra::Point3<f32> = c.into();
        assert_eq!(d, nalgebra::Point3::new(23.1, 5.0, 0.0));

        let v = Vector2::new(23.1, 5.0);

        let b: nalgebra::Point2<f64> = v.into();
        assert_eq!(b, nalgebra::Point2::new(23.1, 5.0));

        let c = Vector2::from(b);
        assert_eq!(c, v);

        let d: nalgebra::Point2<f32> = c.into();
        assert_eq!(d, nalgebra::Point2::new(23.1, 5.0));
    }
}
