use glium::{Surface,glutin};
    use nalgebra_glm as glm;

fn main() {

    //handles window and device events
    let mut event_loop = glutin::event_loop::EventLoop::new();
    //window specific
    let wb = glutin::window::WindowBuilder::new();
    //opengl specific
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    //creates display with above attributes
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut world = glm::translation(&glm::Vec3::new(0.0,0.0,-20.0));
    //let world:[[f32; 4]; 4] = world.into();

    let view = glm::look_at(
        &glm::Vec3::new(0.0,5.0,3.0),//eye position
        &glm::Vec3::new(0.0,0.0,-20.0),//looking at
        &glm::Vec3::new(0.0,1.0,0.0));//up
    let view:[[f32; 4]; 4] = view.into();

    let perspective = glm::perspective(
        4.0 / 3.0, 3.14 / 4.0, 0.01, 10000.0  
    );
    let perspective:[[f32; 4]; 4] = perspective.into();

    let ico = graphics::shapes::icosahedron();

    let positions = glium::VertexBuffer::new(&display,&ico.vertices).unwrap();
    let index_buffer = glium::IndexBuffer::new(&display,glium::index::PrimitiveType::TrianglesList, &ico.indices).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
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
        target.clear_color_and_depth((0.0, 0.25, 0.5, 1.0), 1.0);

        world = glm::rotate_y(&world, -0.002);
        let world_mat:[[f32; 4]; 4] = world.into();

        target.draw(&positions, &index_buffer ,&program,  &glium::uniform! {world:world_mat, view:view, perspective: perspective}, &params).unwrap();

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

mod graphics;
