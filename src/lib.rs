use lazy_static::lazy_static;

pub mod convs;
mod converter;

pub use self::converter::*;
// include!(concat!(env!("OUT_DIR"), "/convs.rs"));

lazy_static! {
    #[allow(non_upper_case_globals)]
    pub static ref Zh2HantConverter: ZhConverter = convs::build_converter(convs::ZH_HANT_CONV);
    pub static ref Zh2HansConverter: ZhConverter = convs::build_converter(convs::ZH_HANS_CONV);
    pub static ref Zh2TWConverter: ZhConverter = convs::build_converter(*convs::ZH_HANT_TW_CONV);
    pub static ref Zh2HKConverter: ZhConverter = convs::build_converter(*convs::ZH_HANT_HK_CONV);
    pub static ref Zh2MOConverter: ZhConverter = convs::build_converter(*convs::ZH_HANT_MO_CONV);
    pub static ref Zh2CNConverter: ZhConverter = convs::build_converter(*convs::ZH_HANS_CN_CONV);
    pub static ref Zh2SGConverter: ZhConverter = convs::build_converter(*convs::ZH_HANS_SG_CONV);
    pub static ref Zh2MYConverter: ZhConverter = convs::build_converter(*convs::ZH_HANS_MY_CONV);
}
