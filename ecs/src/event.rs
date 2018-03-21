pub trait EventTrait: Clone {
    type EventFlag: Default + Clone + Copy;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool;
}

