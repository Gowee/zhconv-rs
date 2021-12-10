use once_cell::unsync::Lazy;
use regex::Regex;

/// Split at the index of the first `needle` if it exists or at the end.
pub fn split_once(haystack: &str, needle: char) -> (&str, &str) {
    haystack.find(needle).map_or_else(
        || (haystack, ""),
        |sc| {
            let (first, last) = haystack.split_at(sc);
            (first, last.split_at(1).1)
        },
    )
}

// #[macro_export]
// macro_rules! impl_eq {
//     ($lhs:ty, $rhs: ty) => {
//         #[stable(feature = "rust1", since = "1.0.0")]
//         #[allow(unused_lifetimes)]
//         impl<'a, 'b> PartialEq<$rhs> for $lhs {
//             #[inline]
//             fn eq(&self, other: &$rhs) -> bool {
//                 PartialEq::eq(&self[..], &other[..])
//             }
//             #[inline]
//             fn ne(&self, other: &$rhs) -> bool {
//                 PartialEq::ne(&self[..], &other[..])
//             }
//         }

//         #[stable(feature = "rust1", since = "1.0.0")]
//         #[allow(unused_lifetimes)]
//         impl<'a, 'b> PartialEq<$lhs> for $rhs {
//             #[inline]
//             fn eq(&self, other: &$lhs) -> bool {
//                 PartialEq::eq(&self[..], &other[..])
//             }
//             #[inline]
//             fn ne(&self, other: &$lhs) -> bool {
//                 PartialEq::ne(&self[..], &other[..])
//             }
//         }
//     };
// }

// static REGEX_HEP: Lazy<Regex> = Lazy::new(|| Regex::new(r"(&[#a-zA-Z0-9]+);").unwrap());  // html entities

// pub fn split_semicolon_allowing_html_entities(s: &str) -> impl Iterator<Item=&str> {
//     let mut es = REGEX_HEP.find_iter(s).map(|m| m.end() - 1).peekable(); // the semicolon indices of entites

//     let mut i = 0;

//     for (j, &c) in s.as_bytes().iter().enumerate() {
//         if c == b';' {
//             // ensure ei >= j
//             while let Some(&ei) = es.peek() {
//                 if ei < j {
//                     es.next();
//                 }
//             }
//             if let Some(ei) = es.peek() {
//                 if ei == j {
//                     continue
//                 }
//                 else {

//                 }
//             }
//         }
//     }

//     unimplemented!();
// }z

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
