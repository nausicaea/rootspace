use lazy_static::lazy_static;
use regex::Regex;

pub fn short_type_name<T: ?Sized>() -> String {
    lazy_static! {
        static ref TPRE: Regex = Regex::new(r"(\w+::)+").unwrap();
    }
    let tn = std::any::type_name::<T>();

    let stn = TPRE.replace_all(tn, "");

    stn.to_string()
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use super::*;

    mod greek {
        pub struct Alpha;
        pub struct Beta<T>(T);
        pub struct Gamma<T, U>(T, U);
    }

    #[test]
    fn shorten_primitive_types() {
        assert_eq!(type_name::<usize>(), "usize");
        assert_eq!(short_type_name::<usize>(), "usize");
    }

    #[test]
    fn shorten_tuple_types() {
        assert_eq!(type_name::<()>(), "()");
        assert_eq!(short_type_name::<()>(), "()");

        assert_eq!(
            type_name::<(usize, greek::Alpha)>(),
            "(usize, ecs::short_type_name::tests::greek::Alpha)"
        );
        assert_eq!(short_type_name::<(usize, greek::Alpha)>(), "(usize, Alpha)");
    }

    #[test]
    fn shorten_array_types() {
        assert_eq!(
            type_name::<[greek::Alpha; 10]>(),
            "[ecs::short_type_name::tests::greek::Alpha; 10]"
        );
        assert_eq!(short_type_name::<[greek::Alpha; 10]>(), "[Alpha; 10]");
    }

    #[test]
    fn shorten_reference_types() {
        assert_eq!(
            type_name::<&greek::Beta<&greek::Alpha>>(),
            "&ecs::short_type_name::tests::greek::Beta<&ecs::short_type_name::tests::greek::Alpha>"
        );
        assert_eq!(short_type_name::<&greek::Beta<&greek::Alpha>>(), "&Beta<&Alpha>");
    }

    #[test]
    fn shorten_simple_types() {
        assert_eq!(type_name::<greek::Alpha>(), "ecs::short_type_name::tests::greek::Alpha");
        assert_eq!(short_type_name::<greek::Alpha>(), "Alpha");
    }

    #[test]
    fn shorten_t1_generics() {
        assert_eq!(
            type_name::<greek::Beta<usize>>(),
            "ecs::short_type_name::tests::greek::Beta<usize>"
        );
        assert_eq!(short_type_name::<greek::Beta<usize>>(), "Beta<usize>");

        assert_eq!(
            type_name::<greek::Beta<greek::Alpha>>(),
            "ecs::short_type_name::tests::greek::Beta<ecs::short_type_name::tests::greek::Alpha>"
        );
        assert_eq!(short_type_name::<greek::Beta<greek::Alpha>>(), "Beta<Alpha>");
    }

    #[test]
    fn shorten_t2_generics() {
        assert_eq!(
            type_name::<greek::Gamma<usize, usize>>(),
            "ecs::short_type_name::tests::greek::Gamma<usize, usize>"
        );
        assert_eq!(short_type_name::<greek::Gamma<usize, usize>>(), "Gamma<usize, usize>");

        assert_eq!(type_name::<greek::Gamma<greek::Alpha, greek::Alpha>>(), "ecs::short_type_name::tests::greek::Gamma<ecs::short_type_name::tests::greek::Alpha, ecs::short_type_name::tests::greek::Alpha>");
        assert_eq!(
            short_type_name::<greek::Gamma<greek::Alpha, greek::Alpha>>(),
            "Gamma<Alpha, Alpha>"
        );
    }

    #[test]
    fn shorten_nasty_generics() {
        assert_eq!(type_name::<greek::Gamma<(greek::Alpha, usize), [greek::Alpha; 10]>>(), "ecs::short_type_name::tests::greek::Gamma<(ecs::short_type_name::tests::greek::Alpha, usize), [ecs::short_type_name::tests::greek::Alpha; 10]>");
        assert_eq!(
            short_type_name::<greek::Gamma<(greek::Alpha, usize), [greek::Alpha; 10]>>(),
            "Gamma<(Alpha, usize), [Alpha; 10]>"
        );
    }
}
