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

//does the same as the above, just return directed edges, as a result returns double the amount, each edge is ordered the same way as its triangle
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
    points[tri[0] as usize]-to_circumcenter
}

//center of mass of triangle
pub fn centroid(points: &Vec<glm::Vec3>,tri: &Vec<u32>)->glm::Vec3{
    tri.iter()
        .map(|i| points[*i as usize])
        .fold(glm::vec3(0.0, 0.0, 0.0), |acc,x| acc+x)
        /3.0       
}

//returns clockwise varient of triangle
pub fn clockwiseify(points: &Vec<glm::Vec3>,mut tri: Vec<u32>)->Vec<u32>{
    let a = points[tri[0] as usize];
    let b = points[tri[1] as usize];
    let c = points[tri[2] as usize];

    let n = glm::cross(&(b-a), &(c-a));

    let centroid = centroid(points, &tri);

    if glm::dot(&n,&centroid)<0.0{
        tri.reverse();
    }
    tri
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
        .map(|edge|  vec![edge.0 as u32,edge.1 as u32,target])
        .flatten()
        .collect()
}

//implementation of the bowyer watson alg, producing indices
//works in 2d on z=0, to be done on local areas of the planet
pub fn bowyer_watson(all_points:&mut Vec<glm::Vec3>,point_indices:&Vec<u32>)->Vec<u32>{
    //create vec of indices
    let mut indices:Vec<u32> = Vec::with_capacity(point_indices.len()*6);

    //first a clockwise super triangle is made encompassing all points 
    all_points.append(&mut vec![
        glm::vec3(0.0, 1_000.0, 0.0),
        glm::vec3(1_000.0, -1_000.0, 0.0),
        glm::vec3(-1_000.0, -1_000.0, 0.0)]);

    //store its indices so it can be removed later
    let super_tri= all_points.len()-3;
    indices.append(&mut vec![super_tri as u32,(super_tri+1) as u32,(super_tri+2) as u32]);

    //for every point, add it and check if it is inside any tris circumcircle
    //if it is, remove those triangles and attach the point to their edges
    for point_no in point_indices{
        //gets points pos
        let point = all_points[*point_no as usize];
        let mut bad_triangles = Vec::with_capacity(indices.len());
        //filter out triangles whos circumcircle contains point, record triangles seperately
        indices = indices.chunks(3)
            .filter(|tri| {
                //check if new point is within the circumcircle
                let circumcenter = circumcenter(all_points,tri.to_vec());
                //println!("circumcenter: {}",circumcenter);
                if circumcenter == glm::vec3(f32::NAN, f32::NAN, f32::NAN){
                    println!(">:(");
                }
                //radius of circumcircle
                let radius = (circumcenter-all_points[tri[0] as usize]).magnitude();
                //if less than radius, point is inside circumcircle
                if (circumcenter-point).magnitude()<radius{
                    //record triangle
                    bad_triangles.append(&mut tri.to_vec());
                    false
                }else{
                    true
                }
            })
            .flatten()
            .map(|x| *x)
            .collect();
        //connect point to hole left by bad triangles
        let mut new_tris = connect_point(bad_triangles,*point_no);
        //add new triangles to triangulation
        indices.append(&mut new_tris);
    }

    //then remove supertri points
    all_points.drain(super_tri..);

    //return triangles, removing those containing super triangle points
    indices.chunks(3)
        //check if any triangle contains index of supertriangle
        .filter(|triangle| !triangle.into_iter().any(|point| point>=&(super_tri as u32)))
        .flatten()
        .map(|x| *x)
        .collect()
}

//flip algorithm to achieve delaunay triangulation, from arbitrary previous one, do not use at large scale
//although has the advantage of being able to specify area being triangulated
pub fn flip_triangulate(points:&Vec<glm::Vec3>, triangulation: &Vec<u32>){
    //go through each triangle and compare with neighbouring triangles
    //if both triangles arent delaunay, flip
    let mut has_flipped = false;
    //compare each tri with every other
    for a_start in (0..triangulation.len()).into_iter().step_by(3){
        let tri_a = &triangulation[a_start..a_start+3];
        for b_start in (0..triangulation.len()).into_iter().step_by(3){
            let tri_b = &triangulation[b_start..b_start+3];
            //the point in tri_b that will be tested against tri_a's circumcircle
            let mut b_point:u32 = 0;
            //skip tri if the two do not share an edge - share two points
            let shared_points:u8 = tri_b.iter()
                .map(|p| {
                    if tri_a.contains(p){
                        1
                    }else{
                        //record point as not shared
                        b_point = *p;
                        0
                    }
                })
                .sum();
            //skip triangle if no shared edges or is just the same triangle
            if shared_points != 2{
                continue;
            }
            //test if two triangles are delaunay- if non shared point in tri_b isnt inside tri_a's circumcircle
            let circumcenter = circumcenter(points, tri_a.into());
            //test if b_point is closer to circumcenter than any point in tri_a- if so than b_point is within circumcircle nad the tris are not delaunay
            if glm::magnitude(&(points[tri_a[0] as usize]-circumcenter)) <= glm::magnitude(&(points[b_point as usize]-circumcenter)){
                //flip tris
                //triangulation[a_start] = 0;
                has_flipped = true;
            }
        }
    }
}

//takes cartesian point on unit sphere, returns it as stereographic, a pole must be specified
pub fn stereographic(point: glm::Vec3,pole: &glm::Vec3)->glm::Vec3{    
    glm::vec3(point.x/(1.0-point.z), point.y/(1.0-point.z), 0.0)
}