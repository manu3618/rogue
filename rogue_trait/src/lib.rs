pub trait EnumCategory {
    fn categories() -> Vec<Self>
    where
        Self: Sized;
}
