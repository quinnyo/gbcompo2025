pub struct Tilemap<T, const DIM_X: usize, const DIM_Y: usize> {
    cells: Vec<T>,
}

impl<T, const DIM_X: usize, const DIM_Y: usize> Tilemap<T, DIM_X, DIM_Y> {
    /// Tilemap dimension in X axis (columns).
    pub const DIM_X: usize = DIM_X;
    /// Tilemap dimension in Y axis (rows).
    pub const DIM_Y: usize = DIM_Y;
    /// Tilemap size as number of cells.
    pub const DIM: usize = Self::DIM_X * Self::DIM_Y;
}

impl<T, const DIM_X: usize, const DIM_Y: usize> Tilemap<T, DIM_X, DIM_Y>
where
    T: Clone,
{
    pub fn get_cells_xy(&self) -> Vec<T> {
        self.cells.clone()
    }

    /// Iterator over tilemap cells in row major order. `(x, y, cell)`
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        (0..Self::DIM).map(|i| (i % Self::DIM_X, i / Self::DIM_Y, &self.cells[i]))
    }

    pub fn xy_set(&mut self, x: usize, y: usize, value: T) {
        self.cells[Self::xy_index(x, y).expect("xy coordinate out of range")] = value;
    }

    pub fn xy(&self, x: usize, y: usize) -> Option<&T> {
        Self::xy_index(x, y).map(|i| &self.cells[i])
    }

    pub fn xy_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        Self::xy_index(x, y).map(|i| &mut self.cells[i])
    }

    pub fn xy_index(x: usize, y: usize) -> Option<usize> {
        if x >= Self::DIM_X || y >= Self::DIM_Y {
            None
        } else {
            Some(y * Self::DIM_X + x)
        }
    }

    pub fn filled(value: T) -> Self
    where
        T: Clone,
    {
        let mut cells: Vec<T> = Vec::new();
        cells.resize(Self::DIM, value);
        Self { cells }
    }

    pub fn new() -> Self
    where
        T: Default,
    {
        let mut cells: Vec<T> = Vec::new();
        cells.resize_with(Self::DIM, Default::default);
        Self { cells }
    }
}

impl<T, const DIM_X: usize, const DIM_Y: usize> Default for Tilemap<T, DIM_X, DIM_Y>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self::new()
    }
}
