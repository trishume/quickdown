extern crate gleam;
extern crate glutin;
extern crate fasternet_common;
extern crate app_units;
extern crate webrender;

mod app;
mod style;

use gleam::gl;
use glutin::GlContext;
use webrender::api::*;
// use webrender::{PROFILER_DBG, RENDER_TARGET_DBG, TEXTURE_CACHE_DBG};

use app::App;

struct Notifier {
    loop_proxy: glutin::EventsLoopProxy,
}

impl Notifier {
    fn new(loop_proxy: glutin::EventsLoopProxy)-> Notifier {
        Notifier {
            loop_proxy,
        }
    }
}

impl RenderNotifier for Notifier {
    fn new_frame_ready(&mut self) {
        #[cfg(not(target_os = "android"))]
        self.loop_proxy.wakeup().unwrap();
    }

    fn new_scroll_frame_ready(&mut self, _composite_needed: bool) {
        #[cfg(not(target_os = "android"))]
        self.loop_proxy.wakeup().unwrap();
    }
}

pub fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_multitouch()
        .with_visibility(false)
        .with_title("Fasternet Client");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl(glutin::GlRequest::GlThenGles {
            opengl_version: (3, 2),
            opengles_version: (3, 0)
        });
    let gl_window = glutin::GlWindow::new(window_builder, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().ok() };

    let gl = match gl::GlType::default() {
        gl::GlType::Gl => unsafe { gl::GlFns::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _) },
        gl::GlType::Gles => unsafe { gl::GlesFns::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _) },
    };

    let (mut width, mut height) = gl_window.get_inner_size_pixels().unwrap();

    let opts = webrender::RendererOptions {
        debug: true,
        precache_shaders: false,
        enable_subpixel_aa: false, // TODO decide
        enable_aa: true,
        device_pixel_ratio: gl_window.hidpi_factor(),
        .. webrender::RendererOptions::default()
    };

    let size = DeviceUintSize::new(width, height);
    let (mut renderer, sender) = webrender::Renderer::new(gl, opts).unwrap();
    let api = sender.create_api();
    let document_id = api.add_document(size);

    let notifier = Box::new(Notifier::new(events_loop.create_proxy()));
    renderer.set_render_notifier(notifier);
    let pipeline_id = PipelineId(0, 0);

    let mut app = App::new(&api,pipeline_id);

    let epoch = Epoch(0);
    let root_background_color = app.bg_color();

    let dpi_scale = gl_window.hidpi_factor();
    let layout_size = LayoutSize::new((width as f32) / dpi_scale, (height as f32) / dpi_scale);
    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
    let mut resources = ResourceUpdates::new();


    app.render(&api, &mut builder, &mut resources, layout_size, pipeline_id, document_id);
    api.set_display_list(
        document_id,
        epoch,
        Some(root_background_color),
        LayoutSize::new(width as f32, height as f32),
        builder.finalize(),
        true,
        resources
    );
    api.set_root_pipeline(document_id, pipeline_id);
    api.generate_frame(document_id, None);

    // let gl_test = support::load(sgl);
    let mut window_visible = false;

    events_loop.run_forever(|event| {
        // println!("{:?}", event);
        match event {
            glutin::Event::WindowEvent { event, .. } => {
                match event {
                    glutin::WindowEvent::Resized(w, h) => {
                        gl_window.resize(w, h);
                        width = w;
                        height = h;
                        let size = DeviceUintSize::new(width, height);
                        let rect = DeviceUintRect::new(DeviceUintPoint::zero(), size);
                        api.set_window_parameters(document_id, size, rect);
                    },
                    glutin::WindowEvent::Closed |
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {virtual_keycode: Some(glutin::VirtualKeyCode::Escape), .. }, ..
                    } => return glutin::ControlFlow::Break,
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::R),
                            state: glutin::ElementState::Pressed, ..
                        }, ..
                    } => {
                        println!("toggling profiler");
                        let mut flags = renderer.get_debug_flags();
                        flags.toggle(webrender::PROFILER_DBG);
                        renderer.set_debug_flags(flags);
                    }
                    _ => (),
                }

                let dpi_scale = gl_window.hidpi_factor();
                let layout_size = LayoutSize::new((width as f32) / dpi_scale, (height as f32) / dpi_scale);
                if app.on_event(event, &api, layout_size, document_id) {
                    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
                    let mut resources = ResourceUpdates::new();

                    app.render(&api, &mut builder, &mut resources, layout_size, pipeline_id, document_id);
                    api.set_display_list(
                        document_id,
                        epoch,
                        Some(root_background_color),
                        layout_size,
                        builder.finalize(),
                        true,
                        resources
                    );
                    api.generate_frame(document_id, None);
                }
            },
            _ => (),
        }

        renderer.update();
        renderer.render(DeviceUintSize::new(width, height)).unwrap();
        gl_window.swap_buffers().ok();
        if !window_visible {
            gl_window.show();
            window_visible = true;
        }
        glutin::ControlFlow::Continue
    });

    renderer.deinit();
}
