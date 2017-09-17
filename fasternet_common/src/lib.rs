extern crate pulldown_cmark;

pub mod markdown;

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
    pub bg: BlockBackground,
    // link_dests: Vec<ResourceLink>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImageBlock {
    pub path: String,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Block {
    Text(TextBlock),
    Image(ImageBlock),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Chunk {
    pub start: u16,
    pub end: u16,
    pub kind: TextKind,
    // link_num: u8,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum BlockBackground {
    NoBackground,
    Code,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TextKind {
    Header1,
    Header2,
    Paragraph,
    ParagraphBold,
    ParagraphItalic,
    ParagraphCode,
    Link,
}
