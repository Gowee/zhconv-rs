#set text(font: "Source Han Serif")

#import "@local/zhconv:0.5.0": convert, convert-content, detect, variants

#align(center, text(22pt, weight: "bold")[
  zhconv-rs 中文简繁及地區詞轉換
])

= Usage
#import "zhconv.typ": convert-content

Supported targets: #variants.join(", "). Detected script of this title: #detect("简繁转换").

#box(stroke: red,
`#convert("柳外輕雷池上雨，雨聲滴碎荷聲", "zh-hans")`
)

#convert-content([
柳外輕雷池上雨
雨聲滴碎荷聲
小樓西角斷虹明
闌乾倚處
待得月華生
], "zh-tw")

#convert-content([
柳外輕雷池上雨 \
雨聲滴碎荷聲 \
小樓西角斷虹明 \
闌干倚處 \
待得月華生 \
], "zh-hans")
