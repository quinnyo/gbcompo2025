#[derive(Debug, Clone)]
pub enum Brush<T> {
    Solid { rep: u8, val: T },
    Terminator,
}

impl<T> Brush<T> {
    pub const BR_TERM: u8 = 0;
    pub const BR_SOLID: u8 = 1;

    pub fn is_terminator(&self) -> bool {
        matches!(self, Brush::Terminator)
    }

    /// Number of tiles emitted by this brush.
    pub fn size(&self) -> usize {
        match self {
            Brush::Solid { rep, .. } => *rep as usize + 1,
            Brush::Terminator => 0,
        }
    }

    pub fn from_datum(val: T) -> Self {
        Self::Solid { rep: 0, val }
    }
}

impl Brush<u8> {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Brush::Solid { rep, val } => vec![Self::BR_SOLID, *rep, *val],
            Brush::Terminator => vec![Self::BR_TERM],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Brushes<T> {
    data: Vec<Brush<T>>,
}

impl<T: Eq + PartialEq> Brushes<T> {
    pub fn last(&self) -> Option<&Brush<T>> {
        self.data.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut Brush<T>> {
        self.data.last_mut()
    }

    pub fn push_literal(&mut self, pushee: T) {
        if let Some(brush) = self.last_mut() {
            match brush {
                Brush::Solid { rep, val } => {
                    if *val == pushee {
                        *rep += 1;
                        return;
                    }
                }
                Brush::Terminator => panic!(),
            }
        }
        self.push(Brush::from_datum(pushee));
    }

    pub fn push(&mut self, b: Brush<T>) {
        assert!(!self.is_terminated());
        self.data.push(b);
    }

    pub fn is_terminated(&self) -> bool {
        self.last().is_some_and(|b| b.is_terminator())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// number of brushes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// number of tiles
    pub fn size(&self) -> usize {
        self.data.iter().map(|b| b.size()).sum()
    }

    pub fn brushes(&self) -> impl Iterator<Item = &Brush<T>> {
        self.data.iter()
    }
}

impl Brushes<u8> {
    pub fn bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for brush in self.brushes() {
            data.append(&mut brush.bytes());
        }
        data
    }

    pub fn rgbasm(&self, code: &mut Vec<String>) {
        code.push(format!(
            "\tdb {}",
            self.bytes()
                .drain(..)
                .map(|b| format!("${:02X}", b))
                .collect::<Vec<String>>()
                .join(",")
        ));
    }
}
