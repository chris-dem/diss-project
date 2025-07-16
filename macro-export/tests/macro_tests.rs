use macro_export::EnumCycle;
use misc_lib::EnumCycle;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, EnumCycle, PartialEq, Eq, PartialOrd, Ord, Default)]
    enum RGB {
        #[default]
        Red,
        Green,
        Blue,
    }

    #[test]
    fn rgb_cycle() {
        let start = RGB::Red;
        assert_eq!(start.toggle().toggle().toggle(), start);
    }

    #[derive(Debug, Clone, Copy, EnumCycle, PartialEq, Eq)]
    enum RedBlack {
        Red,
        Black,
    }


    fn toggle_state<T: EnumCycle + Default>() -> Option<T>{
        Some(T::default().toggle())
    }

    #[test]
    fn rb_cycle() {
        let start = RedBlack::Red;
        assert_eq!(start.toggle().toggle().toggle(), start.toggle());
    }

    #[test]
    fn func_cycle() {
        assert_eq!(toggle_state::<RGB>().unwrap(), RGB::Green);
    }
}
