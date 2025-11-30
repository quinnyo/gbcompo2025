use std::{cmp, fmt, ops};

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct C2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy + Ord> C2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn min_each(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max_each(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl<T> From<(T, T)> for C2<T> {
    fn from(src: (T, T)) -> Self {
        Self { x: src.0, y: src.1 }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for C2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for C2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: ops::Mul<Output = T>> ops::Mul for C2<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for C2<T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<T: Ord> Ord for C2<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl<T: Ord> PartialOrd for C2<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + fmt::Display> fmt::Display for C2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

pub type C2i32 = C2<i32>;
