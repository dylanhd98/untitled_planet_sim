use glium::{Surface,glutin};
use nalgebra_glm as glm;
use std::io::Cursor;
use std::time::{Duration, Instant};

mod planet;
mod graphics;
mod ui;

fn main() {
    //handles window and device events
    let mut event_loop = glutin::event_loop::EventLoop::new();
    //window specific
    let wb = glutin::window::WindowBuilder::new().with_title("Untitled Planet Sim");
    //opengl specific
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    //creates display with above attributes
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    //loads texture to use for planet lookup
    let surface_texture = {
        //loads data from file
        let image = image::load(Cursor::new(&include_bytes!("../resources/images/lookup.png")),
                                image::ImageFormat::Png).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
        //creates compatible image for glium
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        glium::texture::SrgbTexture2d::new(&display, image).unwrap()
    };

    let mut planet = planet::Planet::new(&display,surface_texture,7);

    //creates new camera
    let dimensions = display.get_framebuffer_dimensions();
    let mut cam = graphics::camera::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
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

    let mut frame_time = Instant::now();

    //compiles shaders from files
    let planet_shader = glium::Program::from_source(&display, include_str!("../resources/shaders/planet/vert.glsl"), include_str!("../resources/shaders/planet/frag.glsl"),
    Some(include_str!("../resources/shaders/planet/geom.glsl"))).unwrap();
    let skybox_shader = glium::Program::from_source(&display, include_str!("../resources/shaders/skybox/vert.glsl"), include_str!("../resources/shaders/skybox/frag.glsl"),
     None).unwrap();

    //loop forever until close event
    event_loop.run(move |event, _, control_flow| {

        let delta_time = frame_time.elapsed().as_millis();
        frame_time = Instant::now();
        //println!("deltatime: {}",delta_time);

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
                            //zoom in and out
                            Some(glutin::event::VirtualKeyCode::E)=> cam.pos *= 0.95,
                            Some(glutin::event::VirtualKeyCode::Q)=> cam.pos *= 1.05,

                            //look left and right
                            Some(glutin::event::VirtualKeyCode::A)=> cam.pos = glm::rotate_y_vec3(&cam.pos,-0.05),
                            Some(glutin::event::VirtualKeyCode::D)=> cam.pos = glm::rotate_y_vec3(&cam.pos, 0.05),
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
                //LOGIC
                cam.update_view();
                planet.update(0.001);

                //RENDERING
                //creates buffer to store image in before drawing to window
                let mut target = display.draw();
                //clears buffer for colors and depth
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                //draw planet
                planet.draw(&mut target, &planet_shader, &params, &cam);
                //finish drawing and draws to window
                target.finish().unwrap();
            },
            _ => (),
        }
    });
}
