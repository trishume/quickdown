use webrender::api::*;
use glutin;
use style::{Theme, BuiltTheme, BuiltTextBlock};
use fasternet_common::TextBlock;
use fasternet_common::markdown::parse_markdown;
use std::fs::File;
use std::io::Read;

pub struct App {
    built_theme: BuiltTheme,
    model: Vec<TextBlock>,
    built_model: Vec<BuiltTextBlock>,
    cursor_position: WorldPoint,
}

const WIDTH: f32 = 660.0;

impl App {
    pub fn new(api: &RenderApi) -> Self {
        let theme = Theme::new();
        let built_theme = BuiltTheme::new(&theme, api);
        let model = Self::load_model();
        let built_model = Self::build_model(&model, &built_theme, &api, WIDTH);
        App { built_theme, model, built_model, cursor_position: WorldPoint::new(0.0,0.0) }
    }

    fn build_model(model: &[TextBlock], built_theme: &BuiltTheme, api: &RenderApi, width: f32) -> Vec<BuiltTextBlock> {
        model.iter().map(|block| {
            BuiltTextBlock::new(block, &built_theme, api, width)
        }).collect()
    }

    fn load_model() -> Vec<TextBlock> {
        let mut f = File::open("/Users/tristan/Box/Dev/Projects/xi-mac/xi-editor/doc/crdt.md").unwrap();
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

        let total_height = self.built_model.iter().map(|block| block.size.height + 20.0).sum();

        let bounds = LayoutRect::new(LayoutPoint::zero(), layout_size);
        builder.push_stacking_context(&PrimitiveInfo::new(bounds),
                                      ScrollPolicy::Scrollable,
                                      None,
                                      TransformStyle::Flat,
                                      None,
                                      MixBlendMode::Normal,
                                      Vec::new());
        let content_rect = LayoutRect::new(LayoutPoint::zero(), LayoutSize::new(WIDTH, total_height));
        let clip_id = builder.define_scroll_frame(None,
                                                  content_rect,
                                                  bounds,
                                                  vec![],
                                                  None,
                                                  ScrollSensitivity::ScriptAndInputEvents);
        builder.push_clip_id(clip_id);

        let x = (layout_size.width - WIDTH) / 2.0;
        let mut y = 10.0;
        for block in &self.built_model {
            block.draw(builder, LayoutPoint::new(x, y));
            y += block.size.height + 20.0;
        }

        builder.pop_clip_id();
        builder.pop_stacking_context();
    }

    pub fn on_event(&mut self,
                event: glutin::WindowEvent,
                api: &RenderApi,
                document_id: DocumentId) -> bool {
        match event {
            glutin::WindowEvent::Resized(_w, _h) => return true,
            glutin::WindowEvent::MouseWheel { device_id, delta, phase } => {
                const LINE_HEIGHT: f32 = 38.0;
                let (dx, dy) = match delta {
                    glutin::MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                    glutin::MouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                };

                api.scroll(document_id,
                           ScrollLocation::Delta(LayoutVector2D::new(dx, dy)),
                           self.cursor_position,
                           ScrollEventPhase::Start);
            },
            glutin::WindowEvent::MouseMoved { device_id, position: (x,y) } => {
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
