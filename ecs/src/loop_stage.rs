bitflags! {
    pub struct LoopStage: u8 {
        const UPDATE = 0x01;
        const DYNAMIC_UPDATE = 0x02;
        const RENDER = 0x04;
        const HANDLE_EVENTS = 0x08;
    }
}

