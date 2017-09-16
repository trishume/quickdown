use webrender::api::*;
use glutin;
use style::{Theme, BuiltTheme, BuiltTextBlock};
use fasternet_common::TextBlock;

pub struct App {
    built_theme: BuiltTheme,
    built_block: BuiltTextBlock,
}

impl App {
    pub fn new(api: &RenderApi) -> Self {
        let theme = Theme::new();
        let built_theme = BuiltTheme::new(&theme, api);
        let block = TextBlock::example();
        let built_block = BuiltTextBlock::new(&block, &built_theme, api, 300.0);
        App { built_theme, built_block }
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

        self.built_block.draw(builder, LayoutPoint::new(10.0,10.0));

        builder.pop_stacking_context();
    }

    pub fn on_event(&mut self,
                event: glutin::WindowEvent,
                _api: &RenderApi,
                _document_id: DocumentId) -> bool {
        match event {
            glutin::WindowEvent::Resized(_w, _h) => return true,
            _ => ()
        }

        false
    }

    pub fn bg_color(&self) -> ColorF {
        self.built_theme.bg_color
    }
}
