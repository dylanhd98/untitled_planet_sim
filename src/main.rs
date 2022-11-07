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

    let mut planet = planet::Planet::new(&display,surface_texture,5);

    let dimensions = display.get_framebuffer_dimensions();
    
    //creates new camera
    let mut cam = graphics::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
        glm::vec3(0.0,0.0,5.0), 
        glm::vec3(0.0,0.0,0.0),
        glm::vec3(0.0,1.0,0.0));

    //parameters that specify how rendering takes place
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
        //defines time per frame
        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        //handle window events
        match event {
            //checking for window events
            glutin::event::Event::WindowEvent { event, .. } => 
                match event {
                    //if key pressed
                    glutin::event::WindowEvent::KeyboardInput { device_id, input, is_synthetic }=>{
                        match input.virtual_keycode{
                            Some(glutin::event::VirtualKeyCode::A)=> cam.pos = glm::rotate_y_vec3(&cam.pos,-0.03),
                            Some(glutin::event::VirtualKeyCode::D)=> cam.pos = glm::rotate_y_vec3(&cam.pos,0.03),
                            _=>()
                        }
                    }
                    //closes window if close event
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    },
                    _ => (),
                },

            //once events are handled, this runs
            glutin::event::Event::MainEventsCleared=>{
                cam.update_view();
                //LOGIC
                planet.update(0.001);

                //RENDERING
                //creates buffer to store image in before drawing to window
                let mut target = display.draw();
                //clears buffer
                target.clear_color_and_depth((0.0, 0.01, 0.01, 1.0), 1.0);

                planet.draw(&mut target, &program, &params, &cam);

                //finish drawing and draws to window
                target.finish().unwrap();
            },
            _ => (),
        }
    });
}
