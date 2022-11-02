use glium::{Surface,glutin};
use nalgebra_glm as glm;
use std::io::Cursor;

mod planet;
mod graphics;

fn main() {
    //handles window and device events
    let mut event_loop = glutin::event_loop::EventLoop::new();
    //window specific
    let wb = glutin::window::WindowBuilder::new().with_title("Untitled Planet Sim");
    //opengl specific
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    //creates display with above attributes
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let surface_texture = {
        //loads data from file
        let image = image::load(Cursor::new(&include_bytes!("../resources/images/lookup.png")),
                                image::ImageFormat::Png).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
        //creates compatible image for glium
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        glium::texture::SrgbTexture2d::new(&display, image).unwrap()
    };

    let planet = planet::Planet::new(&display,surface_texture,2);

    let view:[[f32; 4]; 4] = glm::look_at(
        &glm::vec3(0.0,0.0,4.0),//eye position
        &glm::vec3(0.0,0.0,-5.0),//looking at
        &glm::vec3(0.0,1.0,0.0))//up
        .into();

    let perspective:[[f32; 4]; 4] = glm::perspective(
        4.0 / 3.0, 3.14 / 4.0, 0.01, 10000.0)
        .into();

    let cam = graphics::Camera::new(perspective,view);

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        polygon_mode: glium::draw_parameters::PolygonMode::Fill,
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    //gets shaders from files
    let program = glium::Program::from_source(&display, include_str!("../resources/shaders/vert.glsl"), include_str!("../resources/shaders/frag.glsl"), None).unwrap();
    //loop forever until close event
    event_loop.run(move |event, _, control_flow| {

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        //creates buffer to store image in before drawing to window
        let mut target = display.draw();
        //clears buffer
        target.clear_color_and_depth((0.0, 0.01, 0.01, 1.0), 1.0);

        planet.draw(&mut target, &program, &params, &cam);

        //finish drawing and draws to window
        target.finish().unwrap();

        //handle window events
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {

                //closes window if close event
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                },
                _ => (),
            },
            _ => (),
        }
    });
}
