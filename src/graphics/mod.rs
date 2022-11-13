//crates being used

//all the child modules
pub mod skybox;
pub mod camera;
pub mod shapes;

#[derive(Copy, Clone)]
pub struct VertexPos {
    pub position: [f32;3],
}
glium::implement_vertex!(VertexPos,position);

#[derive(Copy, Clone)]
pub struct PosNorm {
    pub position: [f32;3],
    pub normal : [f32;3],
}
glium::implement_vertex!(PosNorm,position,normal);
