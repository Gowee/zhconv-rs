#set text(font: "Source Han Serif")

#align(center, text(22pt, weight: "bold")[
  zhconv-rs 中文简繁及地區詞轉換
])

= Usage
At first:
`#import "@preview/zhconv:0.0.0": zhconv`
// #import "@preview/zhconv:0.0.0": zhconv
#import "zhconv.typ": zhconv

#box(stroke: red,
`#zhconv([
柳外輕雷池上雨
雨聲滴碎荷聲
小樓西角斷虹明
闌乾倚處
待得月華生
], "zh-hans")`
)

#zhconv([
柳外輕雷池上雨
雨聲滴碎荷聲
小樓西角斷虹明
闌乾倚處
待得月華生
], "zh-tw")

#zhconv([
柳外輕雷池上雨 \
雨聲滴碎荷聲 \
小樓西角斷虹明 \
闌干倚處 \
待得月華生 \
], "zh-hans")