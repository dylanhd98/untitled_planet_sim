//external crates
use glium::{Surface,glutin};
use nalgebra_glm as glm;
use std::io::Cursor;
use std::time::{Duration, Instant};

//child modules
mod planet;
mod graphics;


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

    let mut years_per_second = 0.0;
    let mut current:u32 = 0;

    let mut planet = planet::Planet::new(&display,surface_texture,5,1);

    //creates new camera
    let dimensions = display.get_framebuffer_dimensions();
    let mut cam = graphics::camera::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
        glm::vec3(0.0,0.0,5.0), 
        glm::vec3(0.0,0.0,0.0),
        glm::vec3(0.0,1.0,0.0));

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
                            //connectivity testing REMOVE
                            Some(glutin::event::VirtualKeyCode::K)=> {
                                planet.surface.remove_cell(current);
                                current +=1;
                            },
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
                    },

                    //handle resizing
                    glutin::event::WindowEvent::Resized( new_size) =>{
                        cam.update_ratio(new_size.width as f32/ new_size.height as f32);
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

                //EGUI INPUT
                //handles egui input and what results from it
                egui_glium.run(&display, |egui_ctx| {

                    //left side panel for controls
                    egui::SidePanel::left("Left Panel").resizable(false)
                        .show(egui_ctx,|ui| {
                            ui.label("Years Per Second");
                            ui.add(egui::Slider::new(&mut years_per_second, 0.0..=1000.0).logarithmic(true));

                            ui.label("Terrain Scaling");
                            ui.add(egui::Slider::new(&mut planet.render_data.scale, 0.0..=0.25));

                            ui.label("Light Source");
                            egui::ComboBox::from_id_source("lighting")
                                .selected_text(format!("{:?}", planet.render_data.light_pos))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut planet.render_data.light_pos, planet::LightPosition::Sun, "Sun");
                                    ui.selectable_value(&mut planet.render_data.light_pos, planet::LightPosition::Camera, "Camera");
                                    ui.selectable_value(&mut planet.render_data.light_pos, planet::LightPosition::Fixed, "Fixed");
                                }
                            );

                            ui.label("Polygon Mode");
                            egui::ComboBox::from_id_source("polygon")
                                .selected_text(format!("{:?}", params.polygon_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut params.polygon_mode, glium::PolygonMode::Fill, "Fill");
                                    ui.selectable_value(&mut params.polygon_mode, glium::PolygonMode::Line, "Line");
                                    ui.selectable_value(&mut params.polygon_mode, glium::PolygonMode::Point, "Point");
                                }
                            );

                            ui.label("Map Mode");
                            egui::ComboBox::from_id_source("map_mode")
                                .selected_text(format!("{:?}", planet.render_data.map_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Natural, "Natural");
                                    ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Height, "Height");
                                    ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Temperature, "Temperature");
                                    ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Humidity, "Humidity");
                                }
                            );
                        });
                });


                //LOGIC
                //updates camera view based on new pos specified by user input
                cam.update_view();
                
                //updates planet with the specification of how many days pass per frame
                planet.update(delta_time*years_per_second,&display);//quarter year per second, placeholder

                //RENDERING
                //creates buffer to store image in before drawing to window
                let mut target = display.draw();
                //clears buffer for colors and depth
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
                //draw planet
                planet.draw(&mut target, &planet_shader, &params, &cam);
                //planet.draw(&mut target, &map_shader, &params, &cam);
                //draw ui on top of all
                egui_glium.paint(&display, &mut target);
                //finish drawing and draws to window
                target.finish().unwrap();
            },
            _ => (),
        }
    });
}
