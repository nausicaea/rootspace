#[macro_export]
macro_rules! impl_system_group {
    (
        $(#[$outer:meta])*
        pub enum $name:ident<$h:ty, $a:ty, $d:ty, $e:ty, $ef:ty> {
            $(
                $(#[$inner:ident $(args:tt)*])*
                $variant:ident($type:path),
            )+
        }
    ) => {
        impl_system_group! {
            $(#[$outer])*
            (pub) enum $name<$h, $a, $d, $e, $ef> {
                $(
                    $(#[$inner $($args)*])*
                    $variant($type),
                )+
            }
        }
    };
    (
        $(#[$outer:meta])*
        enum $name:ident<$h:ty, $a:ty, $d:ty, $e:ty, $ef:ty> {
            $(
                $(#[$inner:ident $(args:tt)*])*
                $variant:ident($type:path),
            )+
        }
    ) => {
        impl_system_group! {
            $(#[$outer])*
            () enum $name<$h, $a, $d, $e, $ef> {
                $(
                    $(#[$inner $($args)*])*
                    $variant($type),
                )+
            }
        }
    };
    (
        $(#[$outer:meta])*
        ($($vis:tt)*) enum $name:ident<$h:ty, $a:ty, $d:ty, $e:ty, $ef:ty> {
            $(
            $(#[$inner:ident $(args:tt)*])*
            $variant:ident($type:path),
            )+
        }
    ) => {
        $(#[$outer])*
        $($vis)* enum $name {
            $(
            $(#[$inner $($args)*])*
            $variant($type),
            )+
        }

        $(
        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                $name::$variant(value)
            }
        }
        )+

        impl SystemTrait<$h, $a, $d, $e> for $name {
            fn get_stage_filter(&self) -> LoopStage {
                match *self {
                    $(
                    $name::$variant(ref s) => s.get_stage_filter(),
                    )+
                }
            }
            fn get_event_filter(&self) -> $ef {
                match *self {
                    $(
                    $name::$variant(ref s) => s.get_event_filter(),
                    )+
                }
            }
            fn fixed_update(&mut self, db: &mut $d, evt_mgr: &mut $h, aux: &mut $a, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
                match *self {
                    $(
                    $name::$variant(ref mut s) => s.fixed_update(db, evt_mgr, aux, time, delta_time),
                    )+
                }
            }
            fn update(&mut self, db: &mut $d, evt_mgr: &mut $h, aux: &mut $a, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
                match *self {
                    $(
                    $name::$variant(ref mut s) => s.update(db, evt_mgr, aux, time, delta_time),
                    )+
                }
            }
            fn render(&mut self, db: &$d, aux: &mut $a, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
                match *self {
                    $(
                    $name::$variant(ref mut s) => s.render(db, aux, time, delta_time),
                    )+
                }
            }
            fn handle_event(&mut self, db: &mut $d, evt_mgr: &mut $h, aux: &mut $a, event: &$e) -> Result<(), Error> {
                match *self {
                    $(
                    $name::$variant(ref mut s) => s.handle_event(db, evt_mgr, aux, event),
                    )+
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use failure::Error;
    use loop_stage::LoopStage;
    use system::SystemTrait;
    use mock::{MockEvt, MockEvtFlag, MockEvtMgr, MockAux, MockDb, MockSysA, MockSysB};

    impl_system_group! {
        /// This is a doc comment for testing.
        enum SystemGroup<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockEvtFlag> {
            A(MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>),
            B(MockSysB<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>),
        }
    }

    #[test]
    fn system_group_filters() {
        let g = SystemGroup::A(Default::default());
        assert_eq!(g.get_stage_filter(), MockSysA::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default().get_stage_filter());
        let g = SystemGroup::B(Default::default());
        assert_eq!(g.get_stage_filter(), MockSysB::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default().get_stage_filter());

        let g = SystemGroup::A(Default::default());
        assert_eq!(g.get_event_filter(), MockSysA::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default().get_event_filter());
        let g = SystemGroup::B(Default::default());
        assert_eq!(g.get_event_filter(), MockSysB::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default().get_event_filter());
    }

    #[test]
    fn system_group_coersion() {
        let _g: SystemGroup = MockSysA::default().into();
        let _h: SystemGroup = MockSysB::default().into();
    }

    #[test]
    fn system_group_fixed_update() {
        let mut g = SystemGroup::A(Default::default());
        g.fixed_update(&mut Default::default(), &mut Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::A(ref s) => assert_eq!(s.fixed_update_calls, 1),
            _ => unreachable!(),
        }
        let mut g = SystemGroup::B(Default::default());
        g.fixed_update(&mut Default::default(), &mut Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::B(ref s) => assert_eq!(s.fixed_update_calls, 1),
            _ => unreachable!(),
        }
    }

    #[test]
    fn system_group_update() {
        let mut g = SystemGroup::A(Default::default());
        g.update(&mut Default::default(), &mut Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::A(ref s) => assert_eq!(s.update_calls, 1),
            _ => unreachable!(),
        }
        let mut g = SystemGroup::B(Default::default());
        g.update(&mut Default::default(), &mut Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::B(ref s) => assert_eq!(s.update_calls, 1),
            _ => unreachable!(),
        }
    }

    #[test]
    fn system_group_render() {
        let mut g = SystemGroup::A(Default::default());
        g.render(&Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::A(ref s) => assert_eq!(s.render_calls, 1),
            _ => unreachable!(),
        }
        let mut g = SystemGroup::B(Default::default());
        g.render(&Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::B(ref s) => assert_eq!(s.render_calls, 1),
            _ => unreachable!(),
        }
    }

    #[test]
    fn system_group_handle_event() {
        let mut g = SystemGroup::A(Default::default());
        g.handle_event(&mut Default::default(), &mut Default::default(), &mut Default::default(), &MockEvt::TestEventB(0)).unwrap();
        match g {
            SystemGroup::A(ref s) => assert_eq!(s.handle_event_calls, 1),
            _ => unreachable!(),
        }
        let mut g = SystemGroup::B(Default::default());
        g.handle_event(&mut Default::default(), &mut Default::default(), &mut Default::default(), &MockEvt::TestEventB(0)).unwrap();
        match g {
            SystemGroup::B(ref s) => assert_eq!(s.handle_event_calls, 1),
            _ => unreachable!(),
        }
    }
}
