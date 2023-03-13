//external crates
use nalgebra_glm as glm;
use noise::{NoiseFn, Perlin, Seedable};

//use internal crates
use crate::planet::surface::Cell;

#[derive(Debug)]
#[derive(PartialEq)]
enum Side{
    Left,
    Right
}

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

//takes chain and points, then returns triangles connecting point to triangles
pub fn connect_to_chain(){

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
pub fn flip_triangulate(points:&Vec<glm::Vec3>,mut triangulation: Vec<u32>)->Vec<u32>{
    //go through each triangle and compare with neighbouring triangles
    //if both triangles arent delaunay, flip
    let mut has_flipped = true;
    while has_flipped{
        has_flipped = false;
        //compare each tri with every other
        for a_start in (0..triangulation.len()).into_iter().step_by(3){
            for b_start in (0..triangulation.len()).into_iter().step_by(3){
                //turn ranges representing triangles into indices
                let indices:Vec<u32> = [[a_start..a_start+3],[b_start..b_start+3]].concat()
                    .into_iter()
                    .flat_map(|r| r.map(|i| i as u32).collect::<Vec<u32>>())
                    .collect();
                //get all directional edges of two triangles
                let edges = indices_to_directed_edges(&indices);
                //remove any double edges - are internal, counting how many edges are dublicate ones
                //gets points from remaining edges
                let mut duplicate_edges = 0;
                let tri_points:Vec<u32> = edges.iter()
                    .filter(|edge| {
                        //check if any member of edges isnt the flipped varient of current edge
                        if !edges.iter().any(|e| edge == &&(e.1,e.0)) {
                            duplicate_edges+=1;
                            true
                        }
                        else{
                            false
                        }
                    })
                    .flat_map(|e| [e.0 as u32,e.1 as u32])
                    .collect();
                
                //skip triangle if no shared edges or is just the same triangle
                //should be only one shared edge between the two, thus two duplicates
                if duplicate_edges != 2{
                    continue;
                }
                //test if two triangles are delaunay- if non shared point in tri b isnt inside tri a's circumcircle
                let circumcenter = circumcenter(points, tri_points[0..3].to_vec());
                //test if b_point is closer to circumcenter than any point in tri_a- if so than b_point is within circumcircle nad the tris are not delaunay
                if glm::magnitude(&(points[tri_points[0] as usize]-circumcenter)) <= glm::magnitude(&(points[tri_points[3] as usize]-circumcenter)){
                    //flip tris, shift edges over
                    for offset in 0..3{
                        triangulation[a_start+offset] = tri_points[1+offset];
                        triangulation[b_start+offset] = tri_points[(4+offset)%6];
                    }
                    has_flipped = true;
                }
            }
        }
    }
    triangulation
}

//triangulates a y-monotone polygon, given the polygon provided is arranged counter-clockwise
pub fn monotone_poly(points:&Vec<glm::Vec3>, mut polygon: Vec<usize>)->Vec<u32>{
    //triangulation of the inside of polygon
    let mut triangulation:Vec<u32> = Vec::with_capacity((polygon.len()-2)*3);
    //find index of top and bottom of polygon in polygon vec
    let top = (0..polygon.len()).into_iter().max_by(|a,b| 
        points[polygon[*a]].y
        .total_cmp(&points[polygon[*b]].y))
        .unwrap();
    let bottom = (0..polygon.len()).into_iter().min_by(|a,b| 
        points[polygon[*a]].y
        .total_cmp(&points[polygon[*b]].y))
        .unwrap();
    //use this information in combination with the knowledge the polygon is counter clockwise to find which chain it belongs to
    //order all points while storing what chain they belong to
    let mut ordered_points:Vec<(usize,Side)> = (0..polygon.len()).into_iter()
        .map(|i| {//determines which chain it belongs too
            if i>=top&&i<=bottom{
                (polygon[i],Side::Left)
            }else{
                (polygon[i],Side::Right)
            }
        })
        .collect();
    ordered_points.sort_by(|a,b| points[a.0].y.total_cmp(&points[b.0].y));

    //now go through points, if 3 points turn in towards center of triangle, push three as triangle to triangulation if so and remove middle point of three from ordered points
    let mut current_points:Vec<usize> = Vec::with_capacity(polygon.len());
    current_points.push(ordered_points.pop().unwrap().0);
    let second = ordered_points.pop().unwrap();
    current_points.push(second.0);
    let mut last_side = second.1; 
    //go from largest y value to smallest
    let mut temp_count_remove_this_you_fool = 3;
    for point in ordered_points.into_iter().rev(){
        //check if next point is in same chain as previous
        println!("Point {}, current side: {:?}, last side: {:?}",temp_count_remove_this_you_fool,point.1,last_side);
        temp_count_remove_this_you_fool += 1;
        if last_side == point.1{
            //test if angle internal to the polygon between points is <180, if so triangulate 
            //take edge from polygon to be connected to
            let edge:Vec<usize> = current_points.drain(current_points.len()-2..current_points.len()).collect(); 
            //ensure triangle is clockwise
            if point.1 == Side::Left{
                triangulation.append(&mut vec![edge[0] as u32,edge[1] as u32,point.0 as u32]);
            }else{
                triangulation.append(&mut vec![edge[1] as u32,edge[0] as u32,point.0 as u32]);
            }
            //push first point as that is still in polygon
            current_points.push(edge[0]);
        }else{
            //else attach all previous points to new point as triangles
            //get all points along chain
            let chain:Vec<usize> = current_points.drain(0..).collect();
            //go through chain, connecting point to it
            for (a,b) in chain.iter().zip(chain.iter().skip(1)){
                //add tri to triangulation, ensure is counter-clockwise
                if point.1 == Side::Left{
                    triangulation.append(&mut vec![*a as u32,*b as u32,point.0 as u32]);
                }else{
                    triangulation.append(&mut vec![*b as u32,*a as u32,point.0 as u32]);
                }
            }
            //add last chain member back as still part of polgyon
            current_points.push(*chain.last().unwrap());
        }
        //push point and record last side
        current_points.push(point.0);
        last_side = point.1;
    }
    triangulation
}

//takes cartesian point on unit sphere, returns it as stereographic, a pole must be specified
pub fn stereographic(point: glm::Vec3,pole: &glm::Vec3)->glm::Vec3{    
    glm::vec3(point.x/(1.0-point.z), point.y/(1.0-point.z), 0.0)
}