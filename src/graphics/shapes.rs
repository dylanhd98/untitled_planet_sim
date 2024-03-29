//just a bunch of default shapes and operations that can be done one them
use nalgebra_glm as glm;
use std::collections::{HashMap,HashSet};

//used to just flip a tuple so two containing the same numbers are always identical, used in the hashmap during subdivision
fn order_edge(a:u32,b:u32)->(u32,u32){
    if a>b{
        return (a,b)
    }
    (b,a)
}

//non graphics related shape, pure geometry info
pub struct Shape{
    pub vertices:Vec::<glm::Vec3>,
    pub indices:Vec::<u32>
}
impl Shape{
    //new shape from given vertices and indices
    pub fn new(vertices:Vec::<glm::Vec3>,indices:Vec::<u32>)->Shape{
        Shape{
            vertices,
            indices
        }
    }
    //multiplies every vert by number
    pub fn scale(mut self,scale:f32)->Shape{
        self.vertices=self.vertices
            .iter()
            .map(|v|v*scale)
            .collect();
        self
    }
    //makes every vertex unit length
    pub fn normalize(mut self)->Shape{
        //replaces vertices with their normalzed selfs
        self.vertices=self.vertices
            .iter()
            .map(glm::normalize)
            .collect();
        self
    }
    //turns every triangle into 4 smaller ones
    //only works if all indexing is done in the same direction of rotation
    pub fn subdivide(mut self,iterations:u8)->Shape{
        for _ in 0..iterations{
            //indices length is just triangle amount*3
            //thus, new indices will be 4 times as large, 4 times more triangles
            //this is calculated here to prevent constant memory realocation with each push
            let mut new_indices:Vec::<u32> = Vec::with_capacity(self.indices.len()*4);

            //for every triangle edge, calculate midpoint,add to new vertices, store index in dictionary with edge indices as the key
            //if edge midpoint already calculated, skip
            //prevents unnecessary calculation and duplicate midpoints. 
            let mut midpoints = HashMap::<(u32,u32),u32>::new();
            //for every triangle (every group of 3 indices), check if already calculated
            for tri in self.indices.chunks(3){
                for i in 0..3{
                    //order edge so edges with the same points are the same to be used as key
                    let edge = order_edge(tri[i],tri[(i+1)%3]);

                    //if edge isnt in dictionary, calculate midpoint, add to vertices, store index in dictionary
                    if midpoints.get(&edge).is_none() {
                        let mid = (self.vertices[edge.0 as usize]+self.vertices[edge.1 as usize])*0.5;
                        self.vertices.push(mid);//adds midpoint as vertex
                        midpoints.insert(edge, u32::try_from(self.vertices.len()-1).expect("More vertices than datatype can represent"));
                    }
                }

                //all midpoints should be present in dictionary, add new indices
                //TODO:FIND IF PUSHING EACH INDIVIDUALLY INSTEAD OF IN VEC IS MORE EFFICIENT, ALTHOUGH I IMAGINE NOT
                new_indices.append(&mut vec![
                    //middle tri
                    midpoints[&order_edge(tri[0],tri[1])],
                    midpoints[&order_edge(tri[1],tri[2])],
                    midpoints[&order_edge(tri[2],tri[0])],
                    //top tri
                    tri[0],
                    midpoints[&order_edge(tri[0],tri[1])],
                    midpoints[&order_edge(tri[2],tri[0])],
                    //bottom right tri
                    tri[1],
                    midpoints[&order_edge(tri[1],tri[2])],
                    midpoints[&order_edge(tri[0],tri[1])],
                    //bottom left tri
                    tri[2],
                    midpoints[&order_edge(tri[2],tri[0])],
                    midpoints[&order_edge(tri[1],tri[2])],
                ]);
            }
            self.indices = new_indices;
        }
        println!("points: {}, triangles: {}, triangles per point: {}",self.vertices.len(),self.indices.len()/3,(self.indices.len()/3) as f32/self.vertices.len() as f32);
        self
    }
    //creates icosphere
    pub fn icosahedron()->Shape{
        let ratio = (1.0+f32::sqrt(5.0))/2.0;//golden ratio
    
        let vertices:Vec::<glm::Vec3> = vec![
            glm::vec3(-1.0,ratio,0.0),
            glm::vec3(1.0,ratio,0.0),
            glm::vec3(-1.0,-ratio,0.0),
            glm::vec3(1.0,-ratio,0.0),
    
            glm::vec3(0.0,-1.0,ratio),
            glm::vec3(0.0,1.0,ratio),
            glm::vec3(0.0,-1.0,-ratio),
            glm::vec3(0.0,1.0,-ratio),
    
            glm::vec3(ratio, 0.0,-1.0),
            glm::vec3(ratio, 0.0,1.0),
            glm::vec3(-ratio, 0.0,-1.0),
            glm::vec3(-ratio, 0.0,1.0),
        ];
    
        let indices:Vec::<u32> = vec![
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
    //creates cube
    pub fn cube()->Shape{
    
        let vertices:Vec::<glm::Vec3> = vec![
            //front
            glm::vec3(-1.0, -1.0,  1.0),
            glm::vec3(1.0, -1.0,  1.0),
            glm::vec3(1.0,  1.0,  1.0),
            glm::vec3(-1.0,  1.0,  1.0),
    
            //back
            glm::vec3(-1.0, -1.0, -1.0),
            glm::vec3(1.0, -1.0, -1.0),
            glm::vec3(1.0,  1.0, -1.0),
            glm::vec3(-1.0,  1.0, -1.0)
        ];
    
        let indices:Vec::<u32> = vec![
            // front
            0, 1, 2,
            2, 3, 0,
            // right
            1, 5, 6,
            6, 2, 1,
            // back
            7, 6, 5,
            5, 4, 7,
            // left
            4, 0, 3,
            3, 7, 4,
            // bottom
            4, 5, 1,
            1, 0, 4,
            // top
            3, 2, 6,
            6, 7, 3
        ];
    
        Shape{
            vertices,
            indices
        }
    }

    pub fn triangle()->Shape{
        Shape{
            vertices:vec![
                glm::vec3(0.0,1.0,0.0),
                glm::vec3(1.0,-1.0,0.0),
                glm::vec3(-1.0,-1.0,0.0),
            ],
            indices:vec![0,1,2]
        }
    }

    pub fn heart()->Shape{
    
        let vertices:Vec::<glm::Vec3> = vec![
            glm::vec3(0.0,0.5,0.0),

            glm::vec3(0.5,1.0,0.0),
            glm::vec3(-0.5,1.0,0.0),

            glm::vec3(1.0,0.8,0.0),
            glm::vec3(-1.0,0.8,0.0),

            glm::vec3(0.0,-1.0,0.0)
        ];
    
        let indices:Vec::<u32> = vec![
            0,1,3,
            0,4,2,
            5,0,3,
            5,4,0
        ];
    
        Shape{
            vertices,
            indices
        }
    }
}