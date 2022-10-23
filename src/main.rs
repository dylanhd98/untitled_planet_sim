fn main() {
    use glium::{Surface,glutin};
    use nalgebra_glm as glm;

    //handles window and device events
    let mut event_loop = glutin::event_loop::EventLoop::new();
    //window specific
    let wb = glutin::window::WindowBuilder::new();
    //opengl specific
    let cb = glutin::ContextBuilder::new();
    //creates display with above attributes
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let world = glm::translation(&glm::Vec3::new(0.0,0.0,0.0));
    let world:[[f32; 4]; 4] = world.into();

    let view = glm::look_at(
        &glm::Vec3::new(20.0,0.0,0.0),//eye position
        &glm::Vec3::new(0.0,0.0,0.0),//looking at
        &glm::Vec3::new(0.0,1.0,0.0));//up
    let view:[[f32; 4]; 4] = view.into();


    let perspective = glm::perspective(
        16.0 / 9.0, 3.14 / 4.0, 1.0, 10000.0  
    );
    let perspective:[[f32; 4]; 4] = perspective.into();

    let (vertices,indices) = graphics::test();

    let vert_buffer = glium::VertexBuffer::new(&display,&vertices).unwrap();
    let index_buffer = glium::IndexBuffer::new(&display,glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            //test: glium::draw_parameters::DepthTest::IfLess,
            //write: true,
            .. Default::default()
        },
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
        //clears buffer ILOVEU
        target.clear_color(0.0, 0.0, 0.1, 1.0);

        target.draw(&vert_buffer, &index_buffer ,&program,  &glium::uniform! {world:world, view:view, perspective: perspective}, &params).unwrap();

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


//TODO: All of this
mod graphics{
    #[derive(Debug)]
    #[derive(Copy, Clone)]
    pub struct Vertex {
        position: [f32; 3],
    }
    impl Vertex{
        fn new(x:f32,y:f32,z:f32)->Vertex{
            Vertex{
                position:[x,y,z]
            }
        }
    }
    glium::implement_vertex!(Vertex,position);
        
    pub fn icosahedron()->(Vec::<Vertex>,Vec::<u16>){
        let ratio = (1.0+f32::sqrt(5.0))/2.0;//golden ratio

        let verts:Vec::<Vertex> = vec![
            Vertex::new(-1.0,ratio,0.0),
            Vertex::new(1.0,ratio,0.0),
            Vertex::new(-1.0,-ratio,0.0),
            Vertex::new(1.0,-ratio,0.0),

            Vertex::new(0.0,-1.0,ratio),
            Vertex::new(0.0,1.0,ratio),
            Vertex::new(0.0,-1.0,-ratio),
            Vertex::new(0.0,1.0,-ratio),

            Vertex::new(-1.0, 0.0,ratio),
            Vertex::new(1.0, 0.0,ratio),
            Vertex::new(-1.0, 0.0,-ratio),
            Vertex::new(1.0, 0.0,-ratio),
        ];

        let indices:Vec::<u16> = vec![
            0,1,2
        ];

        (verts,indices)
    }

    pub fn test()->(Vec::<Vertex>,Vec::<u16>){

        let verts:Vec::<Vertex> = vec![
            Vertex::new(0.4,0.4,0.0),
            Vertex::new(0.0,0.8,-1.0),
            Vertex::new(-0.8,0.8,0.0),
        ];

        let indices:Vec::<u16> = vec![
            0,1,2
        ];

        (verts,indices)
    }
}
