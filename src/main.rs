//external crates
use glium::{Surface,glutin::{self, event::MouseButton, dpi::{PhysicalPosition, PhysicalSize}}};
use graphics::camera::Camera;
use nalgebra_glm as glm;
use planet::{Planet, GenInfo};
use std::time::{Duration, Instant};

//child modules
#[cfg(test)]
mod tests;
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

    //set time of start of frame
    let mut frame_time = Instant::now();
    //pos of mouse if middle click pressed previous frame, if wasnt in last frame None is stored
    let mut drag_last:Option<glm::Vec2> = None;
    //mouse position as screen coords with top left of screen being -1,-1 and bottom right being 1,1
    let mut mouse_pos:glm::Vec2 = glm::Vec2::zeros();
    //keep track of size of window
    let mut window_res:PhysicalSize<u32> = {
        let dimensions = display.get_framebuffer_dimensions();
        PhysicalSize::new(dimensions.0, dimensions.1)
    };
    

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
        menu_state: menus::MenuState::None,
        iterations: 5,
        seed: 1,
        plate_no: 2,
        axial_tilt: 23.0/180.0,
        lapse_rate:9.8,
        greenhouse_effect: 0.7
    };

    //set starting game state as generating the planet
    let mut game_state = GameState::Generate(default_gen);

    //loop forever until close event
    event_loop.run(move |event, _, control_flow| {
        //defines time per frame
        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        //handle window events
        if let glutin::event::Event::WindowEvent { event, .. } = event{
            //check game state for the events handled differently based on such
            //if generating planet
            if let GameState::Generate(ref gen) = game_state{
                //if key pressed 
                if let glutin::event::WindowEvent::KeyboardInput { device_id:_, input, is_synthetic:_ }=event{
                    match input.virtual_keycode{
                        Some(glutin::event::VirtualKeyCode::Return)=> {
                            //creates new camera
                            let dimensions = display.get_framebuffer_dimensions();
                            let cam = graphics::camera::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
                                glm::vec3(0.0,0.0,5.0), 
                                glm::Vec3::zeros(),
                                glm::Vec3::y());

                            let planet = planet::Planet::new(&display, &gen);
                            game_state= GameState::Playing(planet, cam)},
                        _=>()
                    }
                }
            }
            //if running sim
            else if let GameState::Playing(_,ref mut cam) = game_state{
                //if mouse wheel scrolled, change camera accordingly
                if let glutin::event::WindowEvent::MouseWheel { device_id:_, delta, phase:_, modifiers:_ } = event{
                    if let glutin::event::MouseScrollDelta::LineDelta(_,y) = delta{
                        //zoom 5% according to direction scrolled
                        cam.pos *= 1.0+(y*0.05);
                    }
                    else if let glutin::event::MouseScrollDelta::PixelDelta(pos) = delta{
                        //zoom 0.5% for each pixel scrolled
                        cam.pos *= 1.0+((pos.y as f32)*0.005);
                    }
                }
                //if mouse moves, record new pos 
                else if let glutin::event::WindowEvent::CursorMoved { device_id:_, position, modifiers:_ } = event{
                    //mouse pos but as uv coords, top left of screen being the origin
                    mouse_pos = glm::vec2(position.x as f32/window_res.width as f32, position.y as f32/window_res.height as f32);
                }
                //if mouse input
                else if let glutin::event::WindowEvent::MouseInput { device_id:_, state, button, modifiers:_  } = event{
                    //if mid button pressed
                    if button == MouseButton::Middle && state == glutin::event::ElementState::Pressed{
                        //record current mouse pos
                        drag_last = Some(mouse_pos); 
                    }
                    else if button == MouseButton::Middle && state == glutin::event::ElementState::Released{
                        //make last pos none as no longer being held
                        drag_last = None; 
                    }
                }
                //if key pressed 
                else if let glutin::event::WindowEvent::KeyboardInput { device_id:_, input, is_synthetic:_ }=event{
                    match input.virtual_keycode{
                        //zoom in and out
                        Some(glutin::event::VirtualKeyCode::E)=> cam.pos *= 0.95,
                        Some(glutin::event::VirtualKeyCode::Q)=> cam.pos *= 1.05,

                        //look left and right
                        Some(glutin::event::VirtualKeyCode::A)=> cam.pos = glm::rotate_y_vec3(&cam.pos,-0.05),
                        Some(glutin::event::VirtualKeyCode::D)=> cam.pos = glm::rotate_y_vec3(&cam.pos, 0.05),

                        //look up and down, for these it creates a rotation axis that is tangent to Y and cam.pos
                        Some(glutin::event::VirtualKeyCode::W)=> cam.pos ={
                            let normal = glm::cross(&cam.pos,&glm::Vec3::y());
                            glm::rotate_vec3(&cam.pos, 0.05, &normal)
                        }, 
                        Some(glutin::event::VirtualKeyCode::S)=> cam.pos = {
                            let normal = glm::cross(&cam.pos,&glm::Vec3::y());
                            glm::rotate_vec3(&cam.pos, -0.05, &normal)
                        },
                        _=>()
                    }
                }
                //update camera if window resized and game state is playing
                else if let glutin::event::WindowEvent::Resized(new_size) = event{
                    cam.update_ratio(new_size.width as f32/ new_size.height as f32);
                    window_res = new_size;
                }
            }

            //if window close event, close window :D
            if let glutin::event::WindowEvent::CloseRequested = event{
                *control_flow = glutin::event_loop::ControlFlow::Exit;
            }

            //do egui event
            egui_glium.on_event(&event);
        }

        //once window events handled, run main thing
        else if let glutin::event::Event::MainEventsCleared = event{
            //reset the delta_time at the start of frame
            let delta_time = frame_time.elapsed().as_secs_f32();
            frame_time = Instant::now();

            //creates buffer to store image in before drawing to window
            let mut target = display.draw();
            //clears buffer for colors and depth
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            //generating planet
            if let GameState::Generate(ref gen_info) = game_state{
                //handles egui input and what results from it
                egui_glium.run(&display, |egui_ctx| {
                    menus::planet_create(egui_ctx, &display,&mut game_state);
                });
            }
            //sim running
            else if let GameState::Playing(ref mut planet,ref mut camera) = game_state{
                //handles egui input and what results from it
                egui_glium.run(&display, |egui_ctx| {
                    menus::playing(egui_ctx,&mut params, planet)
                });

                //rotate camera based on how dragged by middle click
                if let Some(last_pos) = drag_last{
                    //get difference between last and current pos for drag
                    let drag = mouse_pos-last_pos;
                    let normal = glm::cross(&camera.pos,&glm::Vec3::y());
                    //rotate camera around origin using drag
                    camera.pos = glm::rotate_vec3(&camera.pos, drag.y*3.14159, &normal);
                    camera.pos = glm::rotate_y_vec3(&camera.pos, -drag.x*3.14159);
                    //record current pos as new
                    drag_last = Some(mouse_pos);
                }

                //updates camera view based on new pos specified by user input
                camera.update_view();
                
                //updates planet with the specification of how many days pass per frame
                planet.update(delta_time, &display);//quarter year per second, placeholder

                //draw planet
                planet.draw(&mut target, &planet_shader, &params, &camera);
                //planet.draw(&mut target, &map_shader, &params, &camera);
            }
            
            //draw ui on top of all
            egui_glium.paint(&display, &mut target);
            //finish drawing and draws to window
            target.finish().unwrap();
        }
    });
}
