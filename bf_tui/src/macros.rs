#[macro_export]
macro_rules! sublayouts {
    ([$($binding:tt),*] = $layout:tt) => {
        let mut _index = 0usize..;
        $(
            let $binding = $layout[_index.next().unwrap()];
        )*
    };
}
