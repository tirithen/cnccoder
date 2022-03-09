use std::{
    f64::{self, consts::PI},
    fmt,
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
                    .map(|Array($($fname,)*)| Self { $($fname: $fname,)* })
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
    #[derive(Default, Debug, PartialEq, Clone, Copy)]
    pub struct Vector2 {
        pub x: f64,
        pub y: f64,
    }
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn min() -> Self {
        Self {
            x: f64::MIN,
            y: f64::MIN,
        }
    }

    pub fn max() -> Self {
        Self {
            x: f64::MAX,
            y: f64::MAX,
        }
    }

    pub fn distance_to(&self, to: Self) -> f64 {
        ((self.x - to.x) * (self.x - to.x) + (self.y - to.y) * (self.y - to.y)).sqrt()
    }

    /// Computes the angle in radians with respect to the positive x-axis
    pub fn angle(&self) -> f64 {
        (-self.x).atan2(-self.y) + PI
    }

    /// Computes the angle in degrees with respect to the positive x-axis
    pub fn angle_degrees(&self) -> f64 {
        self.angle().to_degrees()
    }

    pub fn add_x(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.x += value;
        vector
    }

    pub fn add_y(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.y += value;
        vector
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

as_serde_tuple! {
    #[derive(Default, Debug, PartialEq, Clone, Copy)]
    pub struct Vector3 {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn min() -> Self {
        Self {
            x: f64::MIN,
            y: f64::MIN,
            z: f64::MIN,
        }
    }

    pub fn max() -> Self {
        Self {
            x: f64::MAX,
            y: f64::MAX,
            z: f64::MAX,
        }
    }

    pub fn xy(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    pub fn xz(&self) -> Vector2 {
        Vector2::new(self.x, self.z)
    }

    pub fn yz(&self) -> Vector2 {
        Vector2::new(self.y, self.z)
    }

    pub fn add_x(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.x += value;
        vector
    }

    pub fn add_y(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.y += value;
        vector
    }

    pub fn add_z(&self, value: f64) -> Self {
        let mut vector = *self;
        vector.z += value;
        vector
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Bounds {
    pub min: Vector3,
    pub max: Vector3,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Vector3::max(),
            max: Vector3::min(),
        }
    }
}

impl Bounds {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(x, y, z),
        }
    }

    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    Metric,
    Imperial,
}

impl fmt::Display for Units {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Units::Metric => "mm",
                Units::Imperial => "inches",
            }
        )
    }
}

impl Default for Units {
    fn default() -> Self {
        Units::Metric
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Clockwise,
    Counterclockwise,
}

impl fmt::Display for Direction {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Direction::Clockwise => "clockwise",
                Direction::Counterclockwise => "counterclockwise",
            }
        )
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Clockwise
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
}
