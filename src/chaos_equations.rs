use std::ops::Add;

#[derive(Clone, Copy)]
pub struct Coord {
    pub x:f64,
    pub y:f64,
    pub z:f64
}

impl Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

pub type ChaosEq = fn(&Coord, dt: f64) -> Coord;

pub fn basic_equation(at: &Coord, dt: f64)->Coord {
    return Coord {
        x: at.x + 1.0 * dt,
        y: at.y + 1.0 * dt,
        z: at.z + 1.0 * dt,
    };
}

pub fn lorenz_attractor_equation(at: &Coord, dt: f64)->Coord {
    let ro = 28.0;
    let sigma = 10.0;
    let beta = 8.0/3.0;

    return Coord {
        x: at.x + (sigma * (at.y - at.x)) * dt,
        y: at.y + (at.x * (ro - at.z) - at.z) * dt,
        z: at.z + (at.x * at.y - beta * at.z) * dt,
    };
}