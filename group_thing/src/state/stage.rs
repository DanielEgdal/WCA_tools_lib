pub enum Stage {
    Single { size: usize },
    Split(Vec<Substage>)
}

struct Substage {
    size: usize,
    identifier: char
}

impl Stage {
    pub fn size(&self) -> usize {
        match self {
            Self::Single { size } => *size,
            Self::Split(substages) => substages.iter().map(|s|s.size).sum()
        }
    }
}