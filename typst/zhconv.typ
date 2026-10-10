// zhconv: Chinese variant conversion for Typst.
//
// Public API:
// - convert(text, target): the one unambiguous entry point, str -> str.
// - convert-wikitext(text, target): conversion aware of MediaWiki variant
//   markup (e.g. `-{zh-hans:...}-`); kept separate so the main signature
//   stays free of rule-selection flags.
// - convert-content(doc, target): best-effort traversal of a content tree.
//   Styling on text nodes may be lost, and phrases spanning nodes are
//   converted without cross-node context. Prefer converting plain strings
//   before layout when fidelity matters.
// - detect(text): coarse script detection, returns "zh-Hans" or "zh-Hant".
// - variants: the supported target variant tags.

#let zhconv-wasm = plugin("./zhconv_typst.wasm")

#let variants = ("zh-Hans", "zh-Hant", "zh-TW", "zh-HK", "zh-MO", "zh-CN", "zh-SG", "zh-MY")

// Normalize the target tag when it unambiguously matches a supported variant
// (case- and underscore-insensitive); otherwise pass it through unchanged so
// the engine's own validation (including the identity variant "zh") applies.
#let normalize-variant(target) = {
  let norm = lower(target).replace("_", "-")
  for v in variants {
    if lower(v) == norm { return v }
  }
  target
}

#let convert(text, target) = {
  str(zhconv-wasm.zhconv(bytes(text), bytes(normalize-variant(target)), bytes((0,))))
}

#let convert-wikitext(text, target) = {
  str(zhconv-wasm.zhconv(bytes(text), bytes(normalize-variant(target)), bytes((1,))))
}

#let detect(text) = {
  if zhconv-wasm.is_hans(bytes(text)).at(0) != 0 { "zh-Hans" } else { "zh-Hant" }
}

#let convert-content(document, target) = {
  if type(document) == str {
    convert(document, target)
  }
  else if type(document) == content and document.has("children") { // container
    for child in document.children [
      #convert-content(child, target)
    ]
  }
  else if type(document) == content and document.has("text") { // typical content
    // Best effort: converted as a plain string; styling on the node is not kept.
    convert(document.text, target)
  }
  else if type(document) == content and document.has("body") { // e.g. circle, rect, list
    let args = document.fields()
    let body = convert-content(args.remove("body"), target)
    document.func()(body, ..args)
  }
  else {
    document
  }
}
