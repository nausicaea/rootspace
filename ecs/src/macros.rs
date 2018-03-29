#[macro_export]
macro_rules! impl_system_group {
    (
        enum $name:ident<$h:ty, $a:ty, $d:ty, $e:ty, $ef:ty> {
            $(
            $variant:ident($type:path),
            )+
        }
    ) => {
        enum $name {
            $(
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
            fn update(&mut self, db: &mut $d, evt_mgr: &mut $h, aux: &mut $a, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
                match *self {
                    $(
                    $name::$variant(ref mut s) => s.update(db, evt_mgr, aux, time, delta_time),
                    )+
                }
            }
            fn dynamic_update(&mut self, db: &mut $d, evt_mgr: &mut $h, aux: &mut $a, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
                match *self {
                    $(
                    $name::$variant(ref mut s) => s.dynamic_update(db, evt_mgr, aux, time, delta_time),
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
    use database::DatabaseTrait;
    use event::{EventTrait, EventManagerTrait};

    #[derive(Clone, Debug, PartialEq)]
    pub enum MockEvt {
        TestEventB(u32),
    }

    impl MockEvt {
        fn as_flag(&self) -> MockEvtFlag {
            match *self {
                MockEvt::TestEventB(_) => MockEvtFlag::TEST_EVENT_B,
            }
        }
    }

    bitflags! {
        #[derive(Default)]
        pub struct MockEvtFlag: u8 {
            const TEST_EVENT_B = 0x02;
        }
    }

    impl EventTrait for MockEvt {
        type EventFlag = MockEvtFlag;

        fn matches_filter(&self, flag: Self::EventFlag) -> bool {
            flag.contains(self.as_flag())
        }
    }


    #[derive(Default, Clone, Debug, PartialEq)]
    struct MockAux;

    #[derive(Default, Clone, Debug, PartialEq)]
    struct MockDb;

    impl DatabaseTrait for MockDb {}

    #[derive(Debug, Default, Clone, PartialEq)]
    struct MockEvtMgr;

    impl EventManagerTrait<MockEvt> for MockEvtMgr {
        fn dispatch_later(&mut self, _event: MockEvt) {
        }
        fn handle_events<F>(&mut self, _handler: F) -> Result<bool, Error>
        where
            F: FnMut(&mut Self, &MockEvt) -> Result<bool, Error>,
        {
            Ok(true)
        }
    }

    #[derive(Default)]
    struct SystemA {
        update_calls: usize,
        dynamic_update_calls: usize,
        render_calls: usize,
        handle_event_calls: usize,
    }

    impl SystemTrait<MockEvtMgr, MockAux, MockDb, MockEvt> for SystemA {
        fn get_stage_filter(&self) -> LoopStage {
            Default::default()
        }
        fn get_event_filter(&self) -> MockEvtFlag {
            Default::default()
        }
        fn update(&mut self, _db: &mut MockDb, _evt_mgr: &mut MockEvtMgr, _aux: &mut MockAux, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
            self.update_calls += 1;
            Ok(())
        }
        fn dynamic_update(&mut self, _db: &mut MockDb, _evt_mgr: &mut MockEvtMgr, _aux: &mut MockAux, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
            self.dynamic_update_calls += 1;
            Ok(())
        }
        fn render(&mut self, _db: &MockDb, _aux: &mut MockAux, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
            self.render_calls += 1;
            Ok(())
        }
        fn handle_event(&mut self, _db: &mut MockDb, _evt_mgr: &mut MockEvtMgr, _aux: &mut MockAux, _event: &MockEvt) -> Result<(), Error> {
            self.handle_event_calls += 1;
            Ok(())
        }
    }

    #[derive(Default)]
    struct SystemB {
        update_calls: usize,
        dynamic_update_calls: usize,
        render_calls: usize,
        handle_event_calls: usize,
    }

    impl SystemTrait<MockEvtMgr, MockAux, MockDb, MockEvt> for SystemB {
        fn get_stage_filter(&self) -> LoopStage {
            Default::default()
        }
        fn get_event_filter(&self) -> MockEvtFlag {
            Default::default()
        }
        fn update(&mut self, _db: &mut MockDb, _evt_mgr: &mut MockEvtMgr, _aux: &mut MockAux, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
            self.update_calls += 1;
            Ok(())
        }
        fn dynamic_update(&mut self, _db: &mut MockDb, _evt_mgr: &mut MockEvtMgr, _aux: &mut MockAux, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
            self.dynamic_update_calls += 1;
            Ok(())
        }
        fn render(&mut self, _db: &MockDb, _aux: &mut MockAux, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
            self.render_calls += 1;
            Ok(())
        }
        fn handle_event(&mut self, _db: &mut MockDb, _evt_mgr: &mut MockEvtMgr, _aux: &mut MockAux, _event: &MockEvt) -> Result<(), Error> {
            self.handle_event_calls += 1;
            Ok(())
        }
    }

    impl_system_group! {
        enum SystemGroup<MockEvtMgr, MockAux, MockDb, MockEvt, MockEvtFlag> {
            A(SystemA),
            B(SystemB),
        }
    }

    #[test]
    fn system_group_coersion() {
        let _g: SystemGroup = SystemA::default().into();
        let _h: SystemGroup = SystemB::default().into();
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
    fn system_group_dynamic_update() {
        let mut g = SystemGroup::A(Default::default());
        g.dynamic_update(&mut Default::default(), &mut Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::A(ref s) => assert_eq!(s.dynamic_update_calls, 1),
            _ => unreachable!(),
        }
        let mut g = SystemGroup::B(Default::default());
        g.dynamic_update(&mut Default::default(), &mut Default::default(), &mut Default::default(), &Default::default(), &Default::default()).unwrap();
        match g {
            SystemGroup::B(ref s) => assert_eq!(s.dynamic_update_calls, 1),
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

// #[macro_export]
// macro_rules! impl_group_trait {
//     (
//         $(#[$outer:meta])*
//         pub enum $name:ident {
//             $(
//                 $(#[$inner:ident $(args:tt)*])*
//                 $variant:ident($type:path),
//             )+
//         }
//     ) => {
//         impl_group_trait! {
//             $(#[$outer])*
//             (pub) enum $name {
//                 $(
//                     $(#[$inner $($args)*])*
//                     $variant($type),
//                 )+
//             }
//         }
//     };
//     (
//         $(#[$outer:meta])*
//         enum $name:ident {
//             $(
//                 $(#[$inner:ident $(args:tt)*])*
//                 $variant:ident($type:path),
//             )+
//         }
//     ) => {
//         impl_group_trait! {
//             $(#[$outer])*
//             () enum $name {
//                 $(
//                     $(#[$inner $($args)*])*
//                     $variant($type),
//                 )+
//             }
//         }
//     };
//     (
//         $(#[$outer:meta])*
//         ($($vis:tt)*) enum $name:ident {
//             $(
//                 $(#[$inner:ident $(args:tt)*])*
//                 $variant:ident($type:path),
//             )+
//         }
//     ) => {
//         $(#[$outer])*
//         $($vis)* enum $name {
//             $(
//                 $(#[$inner $($args)*])*
//                 $variant($type),
//             )+
//         }
//
//         use std::mem;
//
//         $(
//         impl From<$type> for $name {
//             fn from(value: $type) -> Self {
//                 $name::$variant(value)
//             }
//         }
//         )+
//
//         impl $crate::database::GroupTrait for $name {
//             fn borrow<T: Any>(&self) -> Option<&T> {
//                 match *self {
//                     $(
//                     $name::$variant(ref i) => Any::downcast_ref(i),
//                     )+
//                 }
//             }
//             fn borrow_mut<T: Any>(&mut self) -> Option<&mut T> {
//                 match *self {
//                     $(
//                     $name::$variant(ref mut i) => Any::downcast_mut(i),
//                     )+
//                 }
//             }
//             fn take<T: Any>(self) -> Result<T, Self> where Self: Sized {
//                 match self {
//                     $(
//                     $name::$variant(i) => if Any::is::<T>(&i) {
//                         Ok(unsafe { mem::transmute::<$type, T>(i) })
//                     } else {
//                         Err(self)
//                     },
//                     )+
//                 }
//             }
//         }
//     };
// }
