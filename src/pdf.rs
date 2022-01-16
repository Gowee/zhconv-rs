/// Support for PDF conversion

use std::io::{Read, Write};
use std::collections::BTreeMap;

use lopdf::{content::Content, Document, Object};
pub use lopdf::{Result, Error};

use crate::ZhConverter;

pub fn convert_pdf_with(orig: impl Read, mut dest: impl Write, converter: &ZhConverter) -> Result<()> {
    // dbg!("!!!!");
    let mut doc = Document::load_from(orig)?;
    // let mut doc = Document::load("../benches/Pooh wiki.pdf")?;
    // dbg!("!!!!!!");
    let page_ids = doc.page_iter().collect::<Vec<_>>(); // bypass borrow rx limit
    for page_id in page_ids.into_iter() {
        // dbg!(page_id);
        let encodings = doc
            .get_page_fonts(page_id)
            .into_iter()
            .map(|(name, font)| (name, font.get_font_encoding().to_owned()))
            .collect::<BTreeMap<Vec<u8>, String>>();
        let mut content = Content::decode(&doc.get_page_content(page_id)?)?;
        convert_page_content_with(&mut content, &encodings, converter)?;
        let modified_content = content.encode()?;
        doc.change_page_content(page_id, modified_content)?;
    }
    doc.save_to(&mut dest)?;
    Ok(())  
}

fn convert_page_content_with(
    content: &mut Content,
    encodings: &BTreeMap<Vec<u8>, String>,
    converter: &ZhConverter,
) -> Result<()> {
    let mut current_encoding = None;
    dbg!("a page");
    for operation in &mut content.operations {
        // dbg!("a opeartion", &operation.operator);
        match operation.operator.as_ref() {
            "Tf" => {
                let current_font = operation
                    .operands
                    .get(0)
                    .ok_or_else(|| Error::Syntax("missing font operand".to_string()))?
                    .as_name()?;
                current_encoding = encodings.get(current_font).map(std::string::String::as_str);
            }
            _ => {
                // dbg!("a Tj", &operation.operands);
                for bytes in operation.operands.iter_mut().flat_map(Object::as_str_mut) {
                    let decoded_text = Document::decode_text(current_encoding, bytes);
                    let converted = dbg!(converter.convert(dbg!(&decoded_text)));
                    *bytes = Document::encode_text(current_encoding, &converted);
                    // info!("{}", decoded_text);
                    // if decoded_text == text {
                    // let encoded_bytes = Document::encode_text(current_encoding, other_text);
                    // *bytes = encoded_bytes;
                    // }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
