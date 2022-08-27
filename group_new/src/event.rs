#[derive(Eq, PartialEq, Clone, Hash, PartialOrd, Ord)]
pub struct Event(usize);

impl Event {
    const VALID_EVENTS: [&'static str; 17] = ["333", "222", "444", "555", "666", "777", "333oh", "333fm", "333bf", "444bf", "555bf", "pyram", "minx", "sq1", "skewb", "333mbf", "clock"];
    pub fn new(event: &str) -> Option<Event> {
        Self::VALID_EVENTS.iter()
            .enumerate()
            .find(|(_, e)| e == &&event)
            .map(|(idx, _)| Event(idx))
    }

    pub fn id(&self) -> &str {
        &Self::VALID_EVENTS[self.0]
    }

    pub fn usize_id(&self) -> usize {
        self.0
    }

    const MAIN_TYPE: [&'static str; 17] = ["average", "average", "average", "average", "average", "average", "average", "average", "single", "single", "single", "average", "average", "average", "average", "single", "average"];
    pub fn main_type(&self) -> &str {
        Self::MAIN_TYPE[self.0]
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, PartialOrd, Ord)]
pub struct ActivityIdentifier {
    pub event: Event,
    pub attempt: Option<usize>
}

impl ActivityIdentifier {
    pub fn new(event: Event, attempt: Option<usize>) -> ActivityIdentifier {
        ActivityIdentifier { event, attempt }
    }

    pub fn from_id(id: &str) -> ActivityIdentifier {
        let mut iter = id.split("-");
        let event = Event::new(iter.next().unwrap()).unwrap();
        let attempt = iter.next().map(|s|s[1..].parse().unwrap());
        Self::new(event, attempt)
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}