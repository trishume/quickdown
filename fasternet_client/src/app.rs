use webrender::api::*;
use glutin;

pub struct App {

}

impl App {
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

        let grid_rows: usize = 10;
        let grid_cols: usize = 10;
        let padding = 10.0;

        let cell_size = LayoutSize::new(
            (layout_size.width-padding) / (grid_cols as f32),
            (layout_size.height-padding) / (grid_rows as f32));
        let rect_size = cell_size - LayoutSize::new(padding, padding);

        for r in 0..grid_rows {
            for c in 0..grid_cols {
                let pt = LayoutPoint::new(
                    padding+(c as f32)*cell_size.width,
                    padding+(r as f32)*cell_size.height);
                let rect = PrimitiveInfo::new(LayoutRect::new(pt, rect_size));
                builder.push_rect(&rect, ColorF::new(1.0, 1.0, 1.0, 1.0));
            }
        }

        builder.pop_stacking_context();
    }

    pub fn on_event(&mut self,
                event: glutin::WindowEvent,
                api: &RenderApi,
                document_id: DocumentId) -> bool {
        match event {
            glutin::WindowEvent::Resized(w, h) => return true,
            _ => ()
        }

        false
    }
}
