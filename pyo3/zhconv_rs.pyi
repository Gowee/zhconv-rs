from typing import Optional, Union, Literal, Sequence, Tuple, TextIO, Callable

# TODO: case-insensitive literal
ZhVariant = Union[
    Literal["zh"],  # dummy for nothing
    Literal["zh-Hant"],
    Literal["zh-Hans"],
    Literal["zh-TW"],
    Literal["zh-HK"],
    Literal["zh-MO"],
    Literal["zh-CN"],
    Literal["zh-SG"],
    Literal["zh-MY"],
    Literal["zh-hant"],
    Literal["zh-hans"],
    Literal["zh-tw"],
    Literal["zh-hk"],
    Literal["zh-mo"],
    Literal["zh-cn"],
    Literal["zh-sg"],
    Literal["zh-my"],
]

def zhconv(text: str, target: ZhVariant, wikitext: bool = False) -> str:
    pass

def make_converter(
    base: Optional[ZhVariant], rules: Union[Sequence[Tuple[str, str]], str, TextIO]
) -> Callable[[str], str]:
    pass

def is_hans(text: str) -> bool:
    pass

def is_hans_confidence(text: str) -> float:
    pass

def infer_variant(text: str) -> ZhVariant:
    pass

def infer_variant_confidence(text: str) -> Sequence[Tuple[ZhVariant, float]]:
    pass
