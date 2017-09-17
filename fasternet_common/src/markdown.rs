use pulldown_cmark::{Parser, Event, Tag};
use std::mem;
use super::*;

fn end_block(blocks: &mut Vec<TextBlock>, cur_text: &mut String, chunks: &mut Vec<Chunk>) {
    let block = TextBlock {
        content: mem::replace(cur_text, String::new()),
        chunks: mem::replace(chunks, Vec::new()),
    };
    blocks.push(block);
}

fn tag_style(tag: &Tag) -> Option<TextKind> {
    match *tag {
        Tag::Paragraph | Tag::CodeBlock(_) | Tag::List(_) => Some(TextKind::Paragraph),
        Tag::Header(1) => Some(TextKind::Header1),
        Tag::Header(_) => Some(TextKind::Header2),
        Tag::Link(_,_) => Some(TextKind::Link),
        Tag::Strong => Some(TextKind::ParagraphBold),
        Tag::Emphasis => Some(TextKind::ParagraphItalic),
        Tag::Code => Some(TextKind::ParagraphCode),
        _ => None,
    }
}

fn add_chunk(chunks: &mut Vec<Chunk>, stack: &mut Vec<TextKind>, last_chunk: &mut usize, len: usize) {
    if stack.is_empty() { return; }
    let chunk = Chunk {
        start: *last_chunk as u16,
        end: len as u16,
        kind: stack.last().unwrap().clone(),
    };
    *last_chunk = len;
    chunks.push(chunk);
}

pub fn parse_markdown(document: &str) -> Vec<TextBlock> {
    let parser = Parser::new(document);

    let mut blocks = Vec::new();
    let mut cur_text = String::new();

    let mut last_chunk = 0;
    let mut chunks = Vec::new();
    let mut stack = Vec::new();
    for event in parser {
        // println!("{:?}", event);
        match event {
            Event::Text(txt) => cur_text.push_str(&txt),
            Event::Start(tag) => {
                if let Some(style) = tag_style(&tag) {
                    if stack.is_empty() {
                        cur_text.clear();
                        chunks.clear();
                        stack.clear();
                        last_chunk = 0;
                    } else {
                        add_chunk(&mut chunks, &mut stack, &mut last_chunk, cur_text.len());
                    }
                    stack.push(style);
                }

                match tag {
                    Tag::Item => cur_text.push_str("- "),
                    _ => (),
                }
            }
            Event::End(tag) => {
                match tag {
                    Tag::Item => cur_text.push_str("\n"),
                    _ => (),
                }

                if let Some(_) = tag_style(&tag) {
                    add_chunk(&mut chunks, &mut stack, &mut last_chunk, cur_text.len());
                    stack.pop();
                }

                if stack.is_empty() {
                    end_block(&mut blocks, &mut cur_text, &mut chunks);
                }
            }
            _ => ()
        }
    }

    // println!("{:?}", blocks);
    blocks
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use super::*;
    #[test]
    fn parse_readme() {
        let mut f = File::open("../Readme.md").unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();

        let _blocks = parse_markdown(&buffer);
    }
}
