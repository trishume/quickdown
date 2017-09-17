use webrender::api::*;
use glutin;
use style::{Theme, BuiltTheme, BuiltTextBlock};
use fasternet_common::TextBlock;
use fasternet_common::markdown::parse_markdown;
use std::fs::File;
use std::io::Read;

pub struct App {
    built_theme: BuiltTheme,
    built_model: Vec<BuiltTextBlock>,
    cursor_position: WorldPoint,
    root_clip: ClipId,
    scroll_offset: LayoutPoint,
    total_height: f32,
}

const WIDTH: f32 = 680.0;
const PADDING: f32 = 20.0;

impl App {
    pub fn new(api: &RenderApi, pipeline_id: PipelineId) -> Self {
        let theme = Theme::new();
        let built_theme = BuiltTheme::new(&theme, api);
        let model = Self::load_model();
        let (built_model, total_height) = Self::build_model(&model, &built_theme, &api, WIDTH);
        let root_clip = ClipId::new(1, pipeline_id);
        let cursor_position = WorldPoint::new(0.0,0.0);
        let scroll_offset = LayoutPoint::zero();
        App { built_theme, built_model, cursor_position, root_clip, scroll_offset, total_height }
    }

    fn build_model(model: &[TextBlock], built_theme: &BuiltTheme, api: &RenderApi, width: f32) -> (Vec<BuiltTextBlock>, f32) {
        let built_model: Vec<BuiltTextBlock> = model.iter().map(|block| {
            BuiltTextBlock::new(block, &built_theme, api, width)
        }).collect();
        let total_height = built_model.iter().map(|block| block.size.height + PADDING).sum();
        (built_model, total_height)
    }

    fn load_model() -> Vec<TextBlock> {
        let mut f = File::open("/Users/tristan/Box/Dev/Projects/xi-mac/xi-editor/doc/crdt-details.md").unwrap();
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
            block.draw(builder, LayoutPoint::new(x, y));
            y += block.size.height + PADDING;
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
