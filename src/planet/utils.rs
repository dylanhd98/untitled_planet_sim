//external crates
use nalgebra_glm as glm;
use noise::{NoiseFn, Perlin, Seedable};

//use internal crates
use crate::planet::surface::Cell;

//handles perlin noise for generating base
pub fn octive_noise(perlin: Perlin, pos:&glm::Vec3, scale:f32, octives:u8, persistance:f32, lacunarity:f32)->f32{
    let mut noise_value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for _o in 0..octives{
        let perlin_value = perlin.get([
            (pos[0]/scale * frequency) as f64,
            (pos[1]/scale *frequency) as f64,
            (pos[2]/scale *frequency) as f64
        ]) as f32;

        noise_value += perlin_value*amplitude;
        amplitude *= persistance;
        frequency *= lacunarity;
    }
    noise_value
}

//get connections of every cell
pub fn indices_to_connections(indices: &Vec<u32>)->Vec<Vec<usize>>{
    //iterate through indices, for every index, store other two in triangle
    let mut connections:Vec::<Vec<usize>> = vec![Vec::with_capacity(6);indices.len()/3];
    
    indices.chunks(3)
        .for_each(|x|//for each triangle
            {
                //adds connections of each vert in triangle
                for i in 0..3{
                    connections[x[i] as usize].push(x[(i+1)%3] as usize);
                }
            }
        );
    connections
}

//gets all edges, like the above but can be used more efficiently i think
pub fn indices_to_edges(indices: &Vec<u32>)->Vec<(usize,usize)>{
    //TODO: FIND MORE EFFICIENT WAY TO DO THIS, IM SURE THERE IS ONE
    //iterate through indices, for every index, store other two in triangle
    let mut edges:Vec::<(usize,usize)> = Vec::with_capacity(indices.len());
    
    indices.chunks(3)
        .for_each(|x|//for each triangle
            {
                //adds connections of each vert in triangle
                for i in 0..3{
                    let edge = (x[i] as usize,x[(i+1)%3] as usize);
                    //since a cell can never connect to itself and out of the two duplicates of an edge only one will ever be ordered, 
                    //checking if ordered before doing anything will ensure the edge is unique
                    if edge.0<edge.1 {
                        edges.push(edge);
                    }
                }
            }
        );
    edges
}

//does the same as the above, just return directed edges, as a result returns double the amount
pub fn indices_to_directed_edges(indices: &Vec<u32>)->Vec<(usize,usize)>{
    //iterate through indices, for every index, store other two in triangle
    let mut edges:Vec::<(usize,usize)> = Vec::with_capacity(indices.len());
    indices.chunks(3)
        .for_each(|x|//for each triangle
            {
                //adds connections of each vert in triangle
                for i in 0..3{
                    edges.push((x[i] as usize,x[(i+1)%3] as usize));
                }
            }
        );
    edges
}

//gets length of an edge between cells
pub fn edge_length(cells: &Vec<Cell>, edge:&(usize,usize))->f32{
    (cells[edge.0].position - 
        cells[edge.1].position)
        .magnitude()
}

//gets circumcenter of triangle
pub fn circumcenter(a: &glm::Vec3,b: &glm::Vec3,c: &glm::Vec3)->glm::Vec3{
    //vectors pointing along triangle edges, and their cross product, for calculation
    let atoc = c-a;
    let atob = b-a;
    let cross = glm::cross(&atoc, &atob);

    //vector pointing to circumcenter
    let to_circumcenter = 
        (glm::cross(&cross, &atob)*atoc.magnitude_squared() + 
        glm::cross(&atoc, &cross)*atob.magnitude_squared())
        /(2.0*cross.magnitude_squared());

    //actual location in space
    a+to_circumcenter
}

//takes surrounding triangles and a target point, returns new traingles all connecting surrounding edges to target
pub fn connect_point(tris:Vec<u32>, target: u32)->Vec<u32>{
    //divide tris into edges
    //filter out shared edges, by filtering out any with a flipped variaent also contained 
    let mut edges = indices_to_directed_edges(&tris);
    edges = edges.iter()
        .filter(|edge| {
            //check if any member of edges isnt the flipped varient of current edge
            !edges.iter()
                .any(|e| edge == &&(e.1,e.0))
        })
        .map(|e| *e)
        .collect();
    
    //return, connecting each valid edge to target
    edges.iter()
        .map(|edge| [edge.0 as u32,edge.1 as u32,target])
        .flatten()
        .collect()
}

//implementation of the bowyer watson alg, producing indices
//works in 2d using a supplied normal, to be done on local areas of the planet
pub fn bowyer_watson(points:&Vec<glm::Vec3>, normal:glm::Vec3){
    //create vec of indices
    let indices:Vec<i32> = Vec::with_capacity(points.len()*6);

    //first a clockwise super triangle is made encompassing all points on the normals plane
    let a = glm::vec3(0.0, 1_000_000.0, 0.0);
    let b = glm::vec3(1_000_000.0, -1_000_000.0, 0.0);
    let c = glm::vec3(-1_000_000.0, -1_000_000.0, 0.0);

    //for

    //return triangles, removing those containing super triangle points

}
