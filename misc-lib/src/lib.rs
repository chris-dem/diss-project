/// A trait for enums that can cycle through their variants.
/// 
/// # Example
/// ```rust
/// use misc_lib::EnumCycle; 
/// #[derive(PartialEq,Debug)]
/// enum Switch { On, Off }
/// 
/// impl EnumCycle for Switch {
///     fn toggle(&self) -> Self {
///         match self {
///             Switch::On => Switch::Off,
///             Switch::Off => Switch::On,
///         }
///     }
/// }
/// 
/// let switch = Switch::On;
/// assert_eq!(switch.toggle(), Switch::Off);
/// ```
pub trait EnumCycle: PartialEq {
    /// Returns the next variant in the cycle.
    fn toggle(&self) -> Self;
}

