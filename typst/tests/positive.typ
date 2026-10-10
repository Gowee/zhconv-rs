#import "@local/zhconv:0.4.0": zhconv, zhconv-str, is-hans-str

#for row in json("reference.json") {
  assert.eq(zhconv(row.text, row.target, wikitext: row.wikitext), row.expected)
}
#assert.eq(zhconv("汉字", "ZH-HANT"), "漢字")
#assert.eq(zhconv-str("汉字", "zh-hant"), "漢字")
#assert.eq(zhconv("漢字", "zh"), "漢字")
#assert.eq(is-hans-str("汉字转换"), true)
#assert.eq(is-hans-str("漢字轉換"), false)
#assert.eq(zhconv([汉字], "zh-hant"), "漢字")
#assert.eq(zhconv(strong([汉字]), "zh-hant"), strong("漢字"))
#assert.eq(zhconv(emph([汉字]), "zh-hant"), emph("漢字"))
#assert.eq(zhconv(42, "zh-hant"), 42)
// Exercise the existing content traversal with containers and layout elements.
#zhconv([汉字 #strong[汉字] #emph[汉字] #linebreak() #box[汉字]], "zh-hant")

#for text in ("软件在服务器上运行", "簡繁混排：后台與後臺", "漢字") {
  assert.eq(zhconv(text, "zh-MO"), zhconv(text, "zh-HK"))
  assert.eq(zhconv(text, "zh-SG"), zhconv(text, "zh-CN"))
  assert.eq(zhconv(text, "zh-MY"), zhconv(text, "zh-CN"))
}
