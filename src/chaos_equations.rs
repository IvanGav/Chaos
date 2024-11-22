use std::ops::Add;

//a bit redundant - just a (x,y,z) object
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

//any equation with this singiture can be used for determining the particle movement
pub type ChaosEq = fn(&Coord, dt: f64) -> Coord;

//boids - todo?

//move with constant speed
pub fn basic_equation(at: &Coord, dt: f64)->Coord {
    return Coord {
        x: at.x,// + 1.0 * dt,
        y: at.y,// + 1.0 * dt,
        z: at.z,// + 1.0 * dt,
    };
}

/*
    Lorenz Attractor Family
*/

//a generalized lorenz attractor equation
pub fn lorenz_attractor_general(at: &Coord, dt: f64, ro: f64, sigma: f64, beta: f64)->Coord {
    return Coord {
        x: at.x + (sigma * (at.y - at.x)) * dt,
        y: at.y + (at.x * (ro - at.z) - at.z) * dt,
        z: at.z + (at.x * at.y - beta * at.z) * dt,
    };
}

//lorenz attractor with standard values for ro, sigma and beta
pub fn lorenz_attractor_standard(at: &Coord, dt: f64)->Coord {
    let ro = 28.0;
    let sigma = 10.0;
    let beta = 8.0/3.0;

    return lorenz_attractor_general(at, dt, ro, sigma, beta);
}

/*
    Rössler Attractor Family
*/

//a generalized rössler attractor equation
pub fn rossler_attractor_general(at: &Coord, dt: f64, a: f64, b: f64, c: f64)->Coord {
    return Coord {
        x: at.x + (- at.y - at.z) * dt,
        y: at.y + (at.x + a * at.y) * dt,
        z: at.z + (b + at.z * (at.x - c)) * dt,
    };
}

pub fn rossler_attractor_variant1(at: &Coord, dt: f64)->Coord {
    let a = 0.1;
    let b = 0.1;
    let c = 14.0;

    return rossler_attractor_general(at, dt, a, b, c);
}

pub fn rossler_attractor_variant2(at: &Coord, dt: f64)->Coord {
    let a = 0.2;
    let b = 0.2;
    let c = 5.7;

    return rossler_attractor_general(at, dt, a, b, c);
}