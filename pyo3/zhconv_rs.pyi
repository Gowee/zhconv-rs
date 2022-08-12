from typing import Optional, Union, Literal

ZhVariant = Union[
    Literal["zh"],
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

def zhconv(text: str, target: ZhVariant, mediawiki: Optional[bool]) -> str:
    pass
