use webrender::api::*;
use glutin;
use style::{Theme, BuiltTheme, BuiltTextBlock, BuiltBlock, BuiltImageBlock};
use fasternet_common::{Block};
use fasternet_common::markdown::parse_markdown;
use std::fs::File;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use rayon::prelude::*;

pub struct App {
    built_theme: BuiltTheme,
    built_model: Vec<BuiltBlock>,
    cursor_position: WorldPoint,
    root_clip: ClipId,
    scroll_offset: LayoutPoint,
    total_height: f32,
}

const WIDTH: f32 = 680.0;
const PADDING: f32 = 20.0;

impl App {
    pub fn new(api: &RenderApi, pipeline_id: PipelineId, path: &str) -> Self {
        let theme = Theme::new();
        let built_theme = BuiltTheme::new(&theme, api);
        let model = Self::load_model(&path);
        let res_folder = Path::new(&path).parent().unwrap();

        let (built_model, total_height) = Self::build_model(&model, &built_theme, &api, WIDTH, &res_folder);
        let root_clip = ClipId::new(1, pipeline_id);
        let cursor_position = WorldPoint::new(0.0,0.0);
        let scroll_offset = LayoutPoint::zero();
        App { built_theme, built_model, cursor_position, root_clip, scroll_offset, total_height }
    }

    fn build_model(model: &[Block], built_theme: &BuiltTheme, api: &RenderApi, width: f32, res_folder: &Path) -> (Vec<BuiltBlock>, f32) {
        let mut total_height = 0.0;
        let mut to_load = Vec::new();
        let mut built_model: Vec<BuiltBlock> = model.iter().map(|block| {
            match *block {
                Block::Text(ref text_block) => {
                    let block = BuiltTextBlock::new(text_block, &built_theme, api, width);
                    total_height += block.size.height + PADDING;
                    BuiltBlock::Text(block)
                },
                Block::Image(ref image_block) => {
                    let block = BuiltImageBlock::new(api);
                    to_load.push((&image_block.path, block.key));
                    BuiltBlock::Image(block)
                },
            }
        }).collect();

        // read all files and decode images (can be in parallel)
        let to_upload: Vec<(ImageKey, ImageDescriptor, ImageData)> =
            to_load.par_iter().map(|&(path, key)| {
                let (descriptor, data) = BuiltImageBlock::load(res_folder, path);
                (key, descriptor, data)
        }).collect();

        // patch in all aspect ratios
        let ratios: HashMap<ImageKey,LayoutSize> = to_upload.iter().map(|&(key, ref descriptor, _)| {
            (key, LayoutSize::new(descriptor.width as f32, descriptor.height as f32))
        }).collect();
        for block in built_model.iter_mut() {
            if let BuiltBlock::Image(ref mut image_block) = *block {
                image_block.dimensions = ratios[&image_block.key];
                total_height += image_block.height(WIDTH) + PADDING;
            }
        }

        // upload all the images to Webrender
        let mut updates = ResourceUpdates::new();
        for (key, descriptor, data) in to_upload.into_iter() {
            updates.add_image(key, descriptor, data, None);
        }
        api.update_resources(updates);

        (built_model, total_height)
    }

    fn load_model(path: &str) -> Vec<Block> {
        let mut f = File::open(path).unwrap();
        // let mut f = File::open("Readme.md").unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();

        parse_markdown(&buffer)
    }

    pub fn render(&mut self,
              _api: &RenderApi,
              builder: &mut DisplayListBuilder,
              _resources: &mut ResourceUpdates,
              layout_size: LayoutSize,
              _pipeline_id: PipelineId,
              _document_id: DocumentId) {
        println!("rendering at size {:?}", layout_size);

        let bounds = LayoutRect::new(LayoutPoint::zero(), layout_size);
        builder.push_stacking_context(&PrimitiveInfo::new(bounds),
                                      ScrollPolicy::Scrollable,
                                      None,
                                      TransformStyle::Flat,
                                      None,
                                      MixBlendMode::Normal,
                                      Vec::new());
        let content_rect = LayoutRect::new(LayoutPoint::zero(), LayoutSize::new(WIDTH, self.total_height));
        builder.define_scroll_frame(
            Some(self.root_clip),
            content_rect,
            bounds,
            vec![],
            None,
            ScrollSensitivity::ScriptAndInputEvents);
        builder.push_clip_id(self.root_clip);

        let x = (layout_size.width - WIDTH) / 2.0;
        let mut y = 10.0;
        for block in &self.built_model {
            match *block {
                BuiltBlock::Text(ref text_block) => {
                    text_block.draw(builder, LayoutPoint::new(x, y));
                    y += text_block.size.height + PADDING;
                }
                BuiltBlock::Image(ref image_block) =>  {
                    image_block.draw(builder, LayoutPoint::new(x, y), WIDTH);
                    y += image_block.height(WIDTH) + PADDING;
                },
            }
        }

        builder.pop_clip_id();
        builder.pop_stacking_context();
    }

    pub fn on_event(&mut self,
                event: glutin::WindowEvent,
                api: &RenderApi,
                layout_size: LayoutSize,
                document_id: DocumentId) -> bool {
        match event {
            glutin::WindowEvent::Resized(_w, _h) => return true,
            glutin::WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                const LINE_HEIGHT: f32 = 38.0;
                let (_dx, dy) = match delta {
                    glutin::MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                    glutin::MouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                };

                // let scroll_states = api.get_scroll_node_state(document_id);
                // let state = scroll_states.iter().find(|l| l.id == self.root_clip).unwrap();
                // let cur_offset = state.scroll_offset;
                self.scroll_offset += LayoutVector2D::new(0.0, -dy);
                let max_y = self.total_height - layout_size.height;
                if self.scroll_offset.y < 0.0 {
                    self.scroll_offset.y = 0.0;
                } else if self.scroll_offset.y > max_y {
                    self.scroll_offset.y = max_y;
                }
                self.scroll_offset.y = self.scroll_offset.y.round();

                api.scroll_node_with_id(document_id, self.scroll_offset,
                    self.root_clip, ScrollClamping::NoClamping);
            },
            glutin::WindowEvent::MouseMoved { device_id: _, position: (x,y) } => {
                self.cursor_position = WorldPoint::new(x as f32, y as f32);
            }
            _ => ()
        }

        false
    }

    pub fn bg_color(&self) -> ColorF {
        self.built_theme.bg_color
    }
}
