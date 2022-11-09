use glium::Surface;

struct Skybox{
    //vertices are only position data
    vertex_buffer: glium::VertexBuffer<super::VertexPos>,
    //indexbuffer is u8 as only 12 triangles
    index_buffer: glium::IndexBuffer<u32>
    //cubemap
}
impl Skybox{
    
    pub fn new(display: &glium::Display)->Skybox{
        let cube = super::Shape::cube();

        let cube_verts: Vec<super::VertexPos> = cube.vertices.iter()
            .map(|v| 
                    super::VertexPos{
                        position:[v.x,v.y,v.z]
                    }
                )
            .collect();
        Skybox{
            vertex_buffer: glium::VertexBuffer::new(display, &cube_verts).unwrap(),
            index_buffer: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &cube.indices).unwrap(),
        }
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam: &super::camera::Camera){

        let pers:[[f32;4];4] = cam.perspective.into();
        let view:[[f32;4];4] = cam.view.into();
        let uniform = glium::uniform! {
            perspective:pers,
            view: view,
            
        };

        target.draw(&self.vertex_buffer,&self.index_buffer,program,&uniform,params).unwrap();
    }
}
