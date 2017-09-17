use webrender::api::*;
use fasternet_common::*;
use std::collections::HashMap;
use app_units::Au;

use std::fs::File;
use std::u32;
use std::ops::Range;
use std::path::PathBuf;
use std::io::{self, Read};

#[derive(Debug, Clone)]
pub struct ChunkStyle {
    color: ColorF,
    size: Au,
    line_height: Au,
    font: usize,
}

pub struct BuiltChunkStyle {
    style: ChunkStyle,
    font_key: FontKey,
    font_instance: FontInstanceKey,
    char_width: f32,
}

pub struct Theme {
    bg_color: ColorF,
    fonts: Vec<&'static str>,
    style_map: HashMap<TextKind, ChunkStyle>,
}

pub struct BuiltTheme {
    pub bg_color: ColorF,
    // fonts: Vec<FontKey>,
    style_map: HashMap<TextKind, BuiltChunkStyle>,
}

pub struct BuiltTextBlock {
    glyphs: Vec<u32>,
    // advances: Vec<f32>,
    chunks: Vec<BuiltChunk>,
    pub size: LayoutSize,
}

#[derive(Debug)]
pub struct BuiltChunk {
    range: Range<usize>,
    char_width: f32,
    height: f32,
    font_instance: FontInstanceKey,
    color: ColorF,
    newline: bool,
}

impl Theme {
    pub fn new() -> Self {
        let mut style_map = HashMap::new();
        style_map.insert(TextKind::Paragraph, ChunkStyle {
            color: ColorF::new(0.39607, 0.48235, 0.5137, 1.0),
            size: Au::from_px(14),
            line_height: Au::from_px(16),
            font: 0,
        });
        style_map.insert(TextKind::ParagraphBold, ChunkStyle {
            color: ColorF::new(0.39607, 0.48235, 0.5137, 1.0),
            size: Au::from_px(14),
            line_height: Au::from_px(16),
            font: 1,
        });
        style_map.insert(TextKind::Link, ChunkStyle {
            color: ColorF::from(ColorU::new( 38, 139, 210, 255)),
            size: Au::from_px(14),
            line_height: Au::from_px(16),
            font: 0,
        });
        style_map.insert(TextKind::Header1, ChunkStyle {
            color: ColorF::from(ColorU::new( 88, 110, 117, 255)),
            size: Au::from_px(25),
            line_height: Au::from_px(27),
            font: 1,
        });
        style_map.insert(TextKind::Header2, ChunkStyle {
            color: ColorF::from(ColorU::new( 88, 110, 117, 255)),
            size: Au::from_px(18),
            line_height: Au::from_px(20),
            font: 1,
        });
        Theme {
            bg_color: ColorF::from(ColorU::new(253, 246, 227, 255)),
            fonts: vec![
                "Roboto_Mono/RobotoMono-Regular.ttf",
                "Roboto_Mono/RobotoMono-Bold.ttf",
                // "Roboto/Roboto-Regular.ttf",
                // "Open_Sans/OpenSans-Regular.ttf",
            ],
            style_map,
        }
    }
}

impl BuiltTheme {
    pub fn new(theme: &Theme, api: &RenderApi) -> BuiltTheme {
        // TODO combine all of these into one resource update
        let fonts: Vec<FontKey> = theme.fonts.iter().map(|res_path| {
            let bytes = Self::read_resource(res_path).unwrap();
            Self::load_font(api, bytes, 0) // TODO understand index
        }).collect();

        let style_map = theme.style_map.iter().map(|(k, style)| {
            let font_key: FontKey = fonts[style.font];
            // TODO don't create redundant instances
            let font_instance = Self::add_font_instance(api, font_key, style.size);
            let char_width = Self::find_char_width(api, font_key, style.size);
            let built = BuiltChunkStyle { style: style.clone(), font_instance, font_key, char_width };
            (k.clone(), built)
        }).collect();

        BuiltTheme {
            bg_color: theme.bg_color,
            // fonts,
            style_map,
        }
    }

    fn read_resource(res_path: &str) -> io::Result<Vec<u8>> {
        let mut path = PathBuf::new();
        path.push("fasternet_client/res");
        path.push(res_path);
        let mut file = File::open(path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn add_font_instance(api: &RenderApi, font_key: FontKey, size: Au) -> FontInstanceKey {
        let key = api.generate_font_instance_key();
        let mut update = ResourceUpdates::new();
        let options = FontInstanceOptions {
            render_mode: FontRenderMode::Subpixel,
        };
        update.add_font_instance(key, font_key, size, Some(options), None);
        api.update_resources(update);
        key
    }

    fn load_font(api: &RenderApi, bytes: Vec<u8>, index: u32) -> FontKey {
        let key = api.generate_font_key();
        let mut update = ResourceUpdates::new();
        update.add_raw_font(key, bytes, index);
        api.update_resources(update);
        key
    }

    fn find_char_width(api: &RenderApi, font_key: FontKey, size: Au) -> f32 {
        let index: u32 = api.get_glyph_indices(font_key, "m")[0].unwrap();

        let font = FontInstance::new(font_key,
                                     size,
                                     ColorF::new(0.0, 0.0, 0.0, 1.0),
                                     FontRenderMode::Subpixel,
                                     SubpixelDirection::Horizontal,
                                     None);
        let mut keys = Vec::new();
        keys.push(GlyphKey::new(index,
                                LayerPoint::zero(),
                                FontRenderMode::Subpixel,
                                SubpixelDirection::Horizontal));
        let metrics = api.get_glyph_dimensions(font, keys);
        metrics[0].unwrap().advance
    }
}

#[derive(Debug)]
enum Token {
    Word(usize),
    Space,
    Newline,
    End,
}

impl BuiltTextBlock {
    pub fn new(block: &TextBlock, theme: &BuiltTheme, api: &RenderApi, width: f32) -> Self {
        let mut indices = Vec::with_capacity(block.content.len());
        // let mut advances = Vec::with_capacity(block.content.len());
        let mut chunks = Vec::with_capacity(block.chunks.len());

        let mut x = 0.0;
        let mut height = 0.0;
        let mut first_chunk = true;
        for chunk in &block.chunks {
            let range = (chunk.start as usize)..(chunk.end as usize);
            let chunk_str = &block.content[range.clone()];
            let style = &theme.style_map[&chunk.kind];

            // even if this gets split, the whole thing is still the same font
            Self::layout_glyphs(api, style.font_key, chunk_str,
                &mut indices);

            Self::build_chunks(
                &mut chunks, &mut height, &mut x,
                range, &chunk_str, &style,
                first_chunk, width,
            );
            first_chunk = false;
        }
        // println!("{:?} - {}", block.content, block.content.len());
        // println!("{:?}", chunks);

        let size = LayoutSize::new(width, height);
        BuiltTextBlock { glyphs: indices, chunks, size }
    }

    fn build_chunks(chunks: &mut Vec<BuiltChunk>, total_height: &mut f32, x: &mut f32, range: Range<usize>,
                    chunk_str: &str, style: &BuiltChunkStyle,  first_chunk: bool, width: f32) {
        let mut flush_line = |range: &mut Range<usize>, newline: bool| {
            // println!("chunk {:?} {:?} {:?}", total_height, range, newline);
            let height = style.style.line_height.to_f32_px();
            chunks.push(BuiltChunk {
                char_width: style.char_width,
                font_instance: style.font_instance,
                color: style.style.color,
                range: range.clone(), newline, height,
            });
            if newline {
                *total_height = *total_height + height;
            }
            range.start = range.end;
        };

        let mut cur_chunk = range.start..range.start;
        let tokens = Self::tokenize(chunk_str);
        // println!("{:?}", tokens);
        let mut newline = first_chunk;
        for token in tokens.into_iter() {
            match token {
                Token::Word(word_len) => {
                    let space_left = width - *x;
                    let chars_left = (space_left / style.char_width) as usize;
                    let doesnt_fit = (space_left < 0.0) || (word_len > chars_left);
                    // println!("s_left={} c_left={} word={} - {:?} {:?}", space_left, chars_left, word_len, &chunk_str[(cur_chunk.start-range.start)..(cur_chunk.end-range.start)], &chunk_str[(cur_chunk.end-range.start)..(cur_chunk.end+word_len-range.start)]);
                    if doesnt_fit {
                        flush_line(&mut cur_chunk, newline); *x = 0.0; newline = true;
                    }

                    cur_chunk.end += word_len;
                    *x += (word_len as f32) * style.char_width;
                },
                Token::Space => {
                    cur_chunk.end += 1;
                    *x += style.char_width;
                }
                Token::Newline => {
                    flush_line(&mut cur_chunk, newline); *x = 0.0; newline = true;
                }
                Token::End => {
                    flush_line(&mut cur_chunk, newline);
                }
            }
        }
    }

    fn tokenize(chunk_str: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut byte_iter = chunk_str.chars();
        'outer: loop {
            // read a single word
            let mut word_len = 0;
            loop {
                match byte_iter.next() {
                    Some(' ') => {
                        if word_len > 0 { tokens.push(Token::Word(word_len)) }
                        tokens.push(Token::Space);
                        break;
                    },
                    Some('\n') => {
                        if word_len > 0 { tokens.push(Token::Word(word_len)) }
                        tokens.push(Token::Newline);
                        break;
                    }
                    Some(_) => word_len += 1,
                    None => {
                        if word_len > 0 { tokens.push(Token::Word(word_len)) }
                        break 'outer;
                    }
                }
            }
        }
        tokens.push(Token::End);
        tokens
    }

    fn layout_glyphs(api: &RenderApi, font_key: FontKey, text: &str,
                        indices_out: &mut Vec<u32>) {
                        // indices_out: &mut Vec<u32>, advances_out: &mut Vec<f32>) {
        // let indices_iter = api.get_glyph_indices(font_key, text).iter()
        //                            .map(|idx| idx.unwrap_or(u32::max_value()));
        let indices: Vec<u32> = api
                            .get_glyph_indices(font_key, text)
                            .iter()
                            .map(|idx| idx.unwrap_or(u32::max_value()))
                            .collect();

        // let font = FontInstance::new(font_key,
        //                              size,
        //                              ColorF::new(0.0, 0.0, 0.0, 1.0),
        //                              FontRenderMode::Subpixel,
        //                              SubpixelDirection::Horizontal,
        //                              None);
        // let mut keys = Vec::new();
        // for glyph_index in &indices {
        //     keys.push(GlyphKey::new(*glyph_index,
        //                             LayerPoint::zero(),
        //                             FontRenderMode::Subpixel,
        //                             SubpixelDirection::Horizontal));
        // }
        // let metrics = api.get_glyph_dimensions(font, keys);

        indices_out.extend(indices.into_iter());

        // let space_advance = size.to_f32_px() / 3.0;
        // let advances_iter = metrics.iter()
        //                            .map(|m| m.map(|dim| dim.advance).unwrap_or(space_advance));
        // advances_out.extend(advances_iter);
    }

    pub fn draw(&self, builder: &mut DisplayListBuilder, origin: LayoutPoint) {
        let mut pt = origin;
        for chunk in &self.chunks {
            pt = self.draw_chunk(builder, pt, chunk, origin.x);
        }
    }

    fn draw_chunk(&self, builder: &mut DisplayListBuilder, mut pt: LayoutPoint, chunk: &BuiltChunk, left: f32) -> LayoutPoint {
        if chunk.range.end == 535 { println!("{:?}", &self.glyphs.len()) }
        let glyphs = &self.glyphs[chunk.range.clone()];

        if chunk.newline {
            pt.y += chunk.height;
            pt.x = left;
        }
        let text_start_x = pt.x;

        // let advances = &self.advances[chunk.range.clone()];
        // let glyphs = glyphs.iter().zip(advances).map(|arg| {
        let glyphs = glyphs.iter().map(|glyph| {
            let gi = GlyphInstance { index: *glyph as u32,
                                     point: pt, };
            // pt.x += *arg.1;
            pt.x += chunk.char_width;
            gi
        }).collect::<Vec<_>>();

        // TODO fix random *1.5
        let rect = LayoutRect::new(LayoutPoint::new(text_start_x, pt.y - chunk.height),
                                   LayoutSize::new((glyphs.len() as f32)*chunk.char_width,chunk.height*1.2));
        let info = LayoutPrimitiveInfo::new(rect);
        let options = GlyphOptions {
            render_mode: FontRenderMode::Subpixel,
        };
        builder.push_text(&info,
             &glyphs,
             chunk.font_instance,
             chunk.color,
             Some(options));
        pt
    }
}
