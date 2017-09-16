#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ResourceId {
    id: usize,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ResourceLink {
    id: ResourceId,
    size: usize,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct PageHeader {
    blocks: Vec<BlockHeader>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BlockHeader {
    id: ResourceLink,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TextBlock {
    pub content: String,
    pub chunks: Vec<Chunk>,
    // link_dests: Vec<ResourceLink>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Chunk {
    pub start: u16,
    pub end: u16,
    pub kind: TextKind,
    // link_num: u8,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TextKind {
    Header1,
    Header2,
    Paragraph,
    ParagraphBold,
    Link,
}

impl TextBlock {
    pub fn example() -> Self {
        let txt = "the quick brown fox jumps over the lazy dog! Even though the dog was testing the text wrapping at the time. The text wrapping was working pretty well by the way, in case you were wondering.";
        let chunks = vec![
            Chunk { start: 0, end: 9, kind: TextKind::Paragraph },
            Chunk { start: 9, end: 16, kind: TextKind::Paragraph },
            Chunk { start: 16, end: 19, kind: TextKind::Link },
            Chunk { start: 19, end: txt.len() as u16, kind: TextKind::Paragraph },
        ];
        TextBlock {
            content: txt.to_owned(),
            chunks,
        }
    }
}
