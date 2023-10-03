#[macro_export]
macro_rules! defaultable_builder {
    ($(#[$attr:meta])? $vis:vis struct $name:ident $(<$lifetime:lifetime>)? {
        $($(#[$field_attr:meta])? $field:ident : $field_ty:ty),+ $(,)?
    }) => {
        $(#[$attr])?
        $vis struct $name $(<$lifetime>)? {
            $(
                $(#[$field_attr])?
                $field: $field_ty,
            )+
        }

        impl $(<$lifetime>)? $name $(<$lifetime>)? {
            $(
                pub fn $field(&mut self, value: $field_ty) -> &mut Self {
                    self.$field = value;
                    self
                }
            )+
        }
    };
}
