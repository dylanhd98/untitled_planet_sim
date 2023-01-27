//external crates
use glium::{Surface,glutin};
use graphics::camera::Camera;
use nalgebra_glm as glm;
use planet::{Planet, GenInfo};
use std::io::Cursor;
use std::time::{Duration, Instant};

//child modules
mod planet;
mod graphics;
mod menus;

//enum discribing the games current state, containing data specific to each
pub enum GameState{
    Generate(planet::GenInfo),
    Playing(Planet,Camera),
}


fn main() {
    //handles window and device events
    let mut event_loop = glutin::event_loop::EventLoop::new();
    //window specific
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(800.0, 450.0))
        .with_title("Untitled Planet Sim");
    //opengl specific
    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(24);

    //creates display with above attributes
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    //set up ui
    let mut egui_glium = egui_glium::EguiGlium::new(&display, &event_loop);

    //parameters that specify how rendering takes place
    let mut params = glium::DrawParameters {
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
    let planet_shader = glium::Program::from_source(&display, 
        include_str!("../resources/shaders/planet/vert.glsl"), 
        include_str!("../resources/shaders/planet/frag.glsl"),
    Some(include_str!("../resources/shaders/planet/geom.glsl"))).unwrap();

    let map_shader = glium::Program::from_source(&display, 
        include_str!("../resources/shaders/map/vert.glsl"), 
        include_str!("../resources/shaders/planet/frag.glsl"),
     None).unwrap();

     //default settings for planet gen
    let default_gen = planet::GenInfo{
        iterations: 5,
        seed: 1,
        plate_no: 2,
        axial_tilt: 0.4084,
        lapse_rate:9.8,
        greenhouse_effect: 0.7
    };

    let mut game_state = GameState::Generate(default_gen);

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
                        match game_state{
                            GameState::Generate(ref gen)=>{
                                match input.virtual_keycode{
                                    Some(glutin::event::VirtualKeyCode::Return)=> {
                                        //creates new camera
                                        let dimensions = display.get_framebuffer_dimensions();
                                        let mut cam = graphics::camera::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
                                            glm::vec3(0.0,0.0,5.0), 
                                            glm::vec3(0.0,0.0,0.0),
                                            glm::vec3(0.0,1.0,0.0));

                                        let planet = planet::Planet::new(&display, &gen);
                                        game_state= GameState::Playing(planet, cam)},
                                    _=>()
                                }
                            }

                            GameState::Playing(_,ref mut cam) =>{
                                match input.virtual_keycode{
                                    //zoom in and out
                                    Some(glutin::event::VirtualKeyCode::E)=> cam.pos *= 0.95,
                                    Some(glutin::event::VirtualKeyCode::Q)=> cam.pos *= 1.05,
        
                                    //look left and right
                                    Some(glutin::event::VirtualKeyCode::A)=> cam.pos = glm::rotate_y_vec3(&cam.pos,-0.05),
                                    Some(glutin::event::VirtualKeyCode::D)=> cam.pos = glm::rotate_y_vec3(&cam.pos, 0.05),
        
                                    //look up and down, for these it creates a rotation axis that is tangent to Y and cam.pos
                                    Some(glutin::event::VirtualKeyCode::W)=> cam.pos ={
                                        let up = glm::vec3(0.0,1.0,0.0);
                                        let normal = glm::cross(&cam.pos,&up);
                                        glm::rotate_vec3(&cam.pos, 0.05, &normal)
                                    }, 
                                    Some(glutin::event::VirtualKeyCode::S)=> cam.pos = {
                                        let up = glm::vec3(0.0,1.0,0.0);
                                        let normal = glm::cross(&cam.pos,&up);
                                        glm::rotate_vec3(&cam.pos, -0.05, &normal)
                                    },
                                    _=>()
                                }
                            }
                            _=>()
                        }
                    },

                    //handle resizing
                    glutin::event::WindowEvent::Resized( new_size) =>{
                        if let GameState::Playing(_,ref mut cam) = game_state{
                            cam.update_ratio(new_size.width as f32/ new_size.height as f32);
                        }
                    },

                    //closes window if close event
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    },

                    _ => {egui_glium.on_event(&event);},//if no other events, do egui event
                },

            //once events are handled, this runs
            glutin::event::Event::MainEventsCleared=>{
                //reset the delta_time at the start of frame
                let delta_time = frame_time.elapsed().as_secs_f32();
                frame_time = Instant::now();

                //creates buffer to store image in before drawing to window
                let mut target = display.draw();
                //clears buffer for colors and depth
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                match game_state{
                    GameState::Generate(ref gen_info)=>{
                        //handles egui input and what results from it
                        egui_glium.run(&display, |egui_ctx| {
                            menus::planet_create(egui_ctx, &display,&mut game_state);
                        });
                    }

                    //while the sim is running
                    GameState::Playing(ref mut planet,ref mut camera)=>{
                        //handles egui input and what results from it
                        egui_glium.run(&display, |egui_ctx| {
                            menus::playing(egui_ctx,&mut params, planet)
                        });

                        //updates camera view based on new pos specified by user input
                        camera.update_view();
                        
                        //updates planet with the specification of how many days pass per frame
                        planet.update(delta_time, &display);//quarter year per second, placeholder

                        //draw planet
                        planet.draw(&mut target, &planet_shader, &params, &camera);
                        //planet.draw(&mut target, &map_shader, &params, &camera);
                    }
                }
                
                //draw ui on top of all
                egui_glium.paint(&display, &mut target);
                //finish drawing and draws to window
                target.finish().unwrap();
            },
            _ => (),
        }
    });
}
