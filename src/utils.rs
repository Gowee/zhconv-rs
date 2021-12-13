macro_rules! get_with_fallback {
    ( $map:expr, $key:expr, $( $others:tt )* ) => {
        $map.get(&$key).or_else(|| get_with_fallback!($map, $($others)* ))
    };
    ( $map:expr, $key:expr ) => {
        $map.get(&$key)
    };
}
pub(crate) use get_with_fallback;

macro_rules! for_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}
pub(crate) use for_wasm;
