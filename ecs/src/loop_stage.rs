bitflags! {
    #[derive(Default)]
    pub struct LoopStage: u8 {
        const FIXED_UPDATE = 0x01;
        const UPDATE = 0x02;
        const RENDER = 0x04;
        const HANDLE_EVENTS = 0x08;
    }
}

