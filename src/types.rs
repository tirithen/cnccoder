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

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Bounds {
    pub min: Vector3,
    pub max: Vector3,
}

impl Bounds {
    pub fn max() -> Self {
        Self {
            min: Vector3::max(),
            max: Vector3::min(),
        }
    }

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

impl Units {
    pub fn to_string_base_unit(&self) -> String {
        match self {
            Self::Metric => "mm".to_string(),
            Self::Imperial => "inch".to_string(),
        }
    }
}

impl Default for Units {
    fn default() -> Self {
        Units::Metric
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Clockwise,
    Counterclockwise,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Clockwise
    }
}
