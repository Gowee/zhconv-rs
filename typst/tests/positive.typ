#import "@local/zhconv:0.5.0": convert, convert-content, convert-wikitext, detect, variants

#for row in json("reference.json") {
  assert.eq(
    if row.wikitext { convert-wikitext(row.text, row.target) } else { convert(row.text, row.target) },
    row.expected,
  )
}
// Target tags are normalized (case- and underscore-insensitive).
#assert.eq(convert("汉字", "ZH-HANT"), "漢字")
#assert.eq(convert("汉字", "zh_hant"), "漢字")
#assert.eq(convert("汉字", "zh-hant"), "漢字")
// Unknown-looking tags pass through to the engine, which keeps accepting the
// identity variant "zh" (returns the text unchanged).
#assert.eq(convert("漢字", "zh"), "漢字")
// detect: coarse script detection.
#assert.eq(detect("汉字转换"), "zh-Hans")
#assert.eq(detect("漢字轉換"), "zh-Hant")
// variants: the advertised tag list.
#assert.eq(variants.len(), 8)
#assert.eq(variants.at(0), "zh-Hans")
// convert-content: best-effort traversal.
#assert.eq(convert-content([汉字], "zh-hant"), "漢字")
#assert.eq(convert-content(strong([汉字]), "zh-hant"), strong("漢字"))
#assert.eq(convert-content(emph([汉字]), "zh-hant"), emph("漢字"))
#assert.eq(convert-content(42, "zh-hant"), 42)
// Exercise the content traversal with containers and layout elements.
#convert-content([汉字 #strong[汉字] #emph[汉字] #linebreak() #box[汉字]], "zh-hant")

// Regional aliases share dictionaries in this build.
#for text in ("软件在服务器上运行", "簡繁混排：后台與後臺", "漢字") {
  assert.eq(convert(text, "zh-MO"), convert(text, "zh-HK"))
  assert.eq(convert(text, "zh-SG"), convert(text, "zh-CN"))
  assert.eq(convert(text, "zh-MY"), convert(text, "zh-CN"))
}
