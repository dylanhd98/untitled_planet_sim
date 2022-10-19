fn main() {
    use glium::{Surface,glutin};

    //handles window and device events
    let mut event_loop = glutin::event_loop::EventLoop::new();
    //window specific
    let wb = glutin::window::WindowBuilder::new();
    //opengl specific
    let cb = glutin::ContextBuilder::new();
    //creates display with above attributes
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    //load shaders from files
    let program = glium::Program::from_source(&display, include_str!("../resources/shaders/vert.glsl"), include_str!("../resources/shaders/frag.glsl"), None).unwrap();

    //loop forever until close event
    event_loop.run(move |event, _, control_flow| {

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        //creates buffer to store image in before drawing to window
        let mut target = display.draw();
        //clears buffer 
        target.clear_color(0.0, 0.0, 0.1, 1.0);
        //finish drawing and draws to window
        target.finish().unwrap();

        //handle window events
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {

                //closes window if close event
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }
    });
}

mod graphics{
    mod shapes{
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
        }
        glium::implement_vertex!(Vertex,position);
        
        //struct describing the simplest shape, only vertices and indices 
        struct base_shape{
            vertices : Vec::<Vertex>,
            indices : Vec::<u16>
        }
        impl base_shape{
            fn new(verts:[Vertex;3])->base_shape{
                base_shape{
                    vertices:Vec::from(verts),
                    indices:vec![2]
                }
            }
        }
    }
}
