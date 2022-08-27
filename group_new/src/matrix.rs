use fixedbitset::FixedBitSet;

#[derive(Debug)]
pub struct CollisionMatrix {
    data: Vec<FixedBitSet>
}

impl CollisionMatrix {
    pub fn new(size: usize) -> CollisionMatrix {
        CollisionMatrix { data: (0..size).map(|_|FixedBitSet::with_capacity(size)).collect() }
    }

    pub fn add_collision(&mut self, a: usize, b: usize) {
        self.data[a].insert(b);
        self.data[b].insert(a);
    }

    pub fn cross_collision(&mut self, collisions: Vec<usize>) {
        let len = collisions.len();
        for i in 0..len - 1 {
            for j in i + 1..len {
                self.add_collision(i, j);
            }
        }
    }

    pub fn does_collide(&self, a: usize, b: usize) -> bool {
        self.data[a][b]
    }

    pub fn collision_set(&self, idx: usize) -> &FixedBitSet {
        &self.data[idx]
    }
}