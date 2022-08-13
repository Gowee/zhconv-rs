from typing import Optional, Union, Literal, Sequence, Tuple, TextIO, Callable

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

def zhconv(text: str, target: ZhVariant, mediawiki: bool = False) -> str:
    pass

def make_converter(
    base: Optional[ZhVariant], rules: Union[Sequence[Tuple[str, str]], str, TextIO]
) -> Callable[[str], str]:
    pass
