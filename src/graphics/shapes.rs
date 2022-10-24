use super::Vertex;

pub struct Shape{
    pub vertices:Vec::<Vertex>,
    pub indices:Vec::<u16>
}
impl Shape{
    pub fn subdivide(&mut self){
        
    }
}

pub fn icosahedron()->Shape{
    let ratio = (1.0+f32::sqrt(5.0))/2.0;//golden ratio

    let vertices:Vec::<Vertex> = vec![
        Vertex::new(-1.0,ratio,0.0),
        Vertex::new(1.0,ratio,0.0),
        Vertex::new(-1.0,-ratio,0.0),
        Vertex::new(1.0,-ratio,0.0),

        Vertex::new(0.0,-1.0,ratio),
        Vertex::new(0.0,1.0,ratio),
        Vertex::new(0.0,-1.0,-ratio),
        Vertex::new(0.0,1.0,-ratio),

        Vertex::new(ratio, 0.0,-1.0),
        Vertex::new(ratio, 0.0,1.0),
        Vertex::new(-ratio, 0.0,-1.0),
        Vertex::new(-ratio, 0.0,1.0),
    ];

    let indices:Vec::<u16> = vec![
        //tris surrounding point 0
        0,11,5,
        0,5,1,
        0,1,7,
        0,7,10,
        0,10,11,
        //tris connected to above
        1,5,9,
        5,11,4,
        11,10,2,
        10,7,6,
        7,1,8,
        //tris surrounding 3
        3,9,4,
        3,4,2,
        3,2,6,
        3,6,8,
        3,8,9,
        //tris connected to above
        4,9,5,
        2,4,11,
        6,2,10,
        8,6,7,
        9,8,1
    ];

    Shape{
        vertices,
        indices
    }

}

pub fn cube()->Shape{

    let vertices:Vec::<Vertex> = vec![
        Vertex::new(1.0,1.0,1.0),
        Vertex::new(-1.0,1.0,1.0),
        Vertex::new(1.0,-1.0,1.0),
        Vertex::new(-1.0,-1.0,1.0),

        Vertex::new(1.0,1.0,-1.0),
        Vertex::new(-1.0,1.0,-1.0),
        Vertex::new(1.0,-1.0,-1.0),
        Vertex::new(-1.0,-1.0,-1.0)
    ];

    let indices:Vec::<u16> = vec![
        //front face
        0,1,2,
        3,2,1,

        //back face
        6,5,4,
        5,6,7,

        //top face
        0,5,1,
        0,4,5,

        //bottom face
        3,7,2,
        7,6,3,

        //left face
        5,3,1,
        5,7,3,
        
        //right face
        0,2,4,
        2,6,4,
    ];

    Shape{
        vertices,
        indices
    }
}

pub fn tetrahedron(){

}