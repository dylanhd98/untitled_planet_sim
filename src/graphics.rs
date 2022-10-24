pub mod shapes;
    
    #[derive(Copy, Clone)]
    pub struct Vertex {
        position: (f32,f32,f32),
    }
    impl Vertex{
        fn new(x:f32,y:f32,z:f32)->Vertex{
            Vertex{
                position:(x,y,z)
            }
        }
    }
    glium::implement_vertex!(Vertex,position);

    #[derive(Copy, Clone)]
    pub struct Normal {
        normal: (f32, f32, f32)
    }
    glium::implement_vertex!(Normal, normal);