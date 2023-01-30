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
    //for each triangle
    indices.chunks(3)
        .for_each(|x|{
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
    //for each triangle
    indices.chunks(3)
        .for_each(|x|{
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
    indices.chunks(3)//for each triangle
        .for_each(|x|{
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
pub fn circumcenter(points: &Vec<glm::Vec3>, tri: Vec<u32>)->glm::Vec3{
    //vectors pointing along triangle edges, and their cross product, for calculation
    let atoc = points[tri[2] as usize]-points[tri[0] as usize];
    let atob = points[tri[1] as usize]-points[tri[0] as usize];
    let cross = glm::cross(&atoc, &atob);

    //vector pointing to circumcenter
    let to_circumcenter = 
        (glm::cross(&cross, &atob)*atoc.magnitude_squared() + 
        glm::cross(&atoc, &cross)*atob.magnitude_squared())
        /(2.0*cross.magnitude_squared());

    //actual location in space
    points[tri[0] as usize]+to_circumcenter
}

//takes surrounding triangles and a target point, returns new traingles all connecting surrounding edges to target
pub fn connect_point(tris:Vec<u32>, target: u32)->Vec<u32>{
    //divide tris into edges
    //filter out shared edges, by filtering out any with a flipped variaent also contained in edges
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
pub fn bowyer_watson(all_points:&mut Vec<glm::Vec3>,point_indices:&Vec<u32>)->Vec<u32>{
    //create vec of indices
    let mut indices:Vec<u32> = Vec::with_capacity(point_indices.len()*6);

    //first a clockwise super triangle is made encompassing all points on the normals plane
    all_points.append(&mut vec![
        glm::vec3(0.0, 1_000.0, 0.0),
        glm::vec3(1_000.0, -1_000.0, 0.0),
        glm::vec3(-1_000.0, -1_000.0, 0.0)]);

    //store its indices so it can be removed later
    let mut super_tri= all_points.len()-3..all_points.len();

    indices.append(&mut vec![(all_points.len()-3) as u32,(all_points.len()-2) as u32,(all_points.len()-1) as u32]);

    //for every point, add it and check if it is inside any tris circumcircle
    //if it is, remove those triangles and attach the point to their edges
    for point_no in 0..point_indices.len(){
        println!("points_no: {}\npoint_indices: {}\n",point_no,point_indices.len());
        let point = all_points[point_no];
        let mut bad_triangles = Vec::with_capacity(indices.len());
        //filter out triangles whos circumcircle contains point, record triangles seperately
        indices = indices.chunks(3)
            .filter(|tri| {
                //check if new point is within the circumcircle
                let circumcenter = circumcenter(all_points,tri.to_vec());
                //radius of circumcircle
                let radius = (circumcenter-all_points[tri[0] as usize]).magnitude();
                println!("circumcenter: {} \nradius: {} \npoint pos: {}",circumcenter,radius,all_points[point_no]);
                //if less than radius, point is inside circumcircle
                if (circumcenter-point).magnitude()<radius{
                    //record triangle
                    println!("triangle recorded!");
                    bad_triangles.append(&mut tri.to_vec());
                    false
                }else{
                    true
                }
            })
            .flatten()
            .map(|x| *x)
            .collect();
        println!("bad triangles:{}",bad_triangles.len());
        //connect point to hole left by bad triangles
        indices.append(&mut connect_point(bad_triangles,point_no as u32));
        println!("triangle count: {}",indices.len()/3);
    }

    

    //then remove supertri points
    all_points.drain(super_tri.clone());
    //return triangles, removing those containing super triangle points
    indices.chunks(3)
        //checks if any vert in supertri is contained within each triangle
        .filter(|tri| !super_tri.any(|vert| tri.contains(&(vert as u32))))
        .flatten()
        .map(|x| *x)
        .collect()
}

//flipping algorithm for delaunay triangulation
//VERY INEFFICIENT, DO NOT ACTUALLY USE, JUST FOR TESTING
pub fn delaunay_flip(points:&Vec<glm::Vec3>, indices: Vec<u32>)->Vec<u32>{
    //go through each triangle, if opposite angles sums are >180, flip them
    //done until none to flip
    loop{
        for tri in indices.chunks(3){
            //for every tri it borders, preform the check and flip if needed
            let bordering:Vec<&[u32]> = indices.chunks(3)
                .filter(|c| c.contains(&tri[0]) as u8 +c.contains(&tri[1]) as u8+c.contains(&tri[2]) as u8>=2)//check if shares two or more points, then they neighbor
                .collect();

            //compare opposite angles in triangles
            for border_tri in bordering{
                
            }
        }
        return indices;
    }
}
