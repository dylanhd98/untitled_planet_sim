//managing all the planets systems
//implimented on surface in new module for better structuring

//external crates
use nalgebra_glm as glm;
//internal modules
use super::{SimInfo,utils::*};

impl super::surface::Surface{
    //handles tempereture updating
    pub fn temperature(&mut self,years:f32,sim_info: &SimInfo){
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&sim_info.to_sun, &sim_info.axis);

        //updates temp for each
        for cell in self.cells.iter_mut(){
            //amount of light recieved as percentage compared to ideal
            //calculates latitude and gets its distance from the ideal/max 
            let light_angle_multiplier = glm::max2_scalar(1.0-f32::abs(sun_max- glm::dot(&cell.position,&sim_info.axis)), 0.0);
            //multiplies ideal temp by angle, then takes lapse rate*height away if above sea level
            cell.contents.temperature = ((40.0*light_angle_multiplier)-(glm::max2_scalar(cell.contents.height,0.0)*sim_info.lapse_rate))*sim_info.greenhouse_effect;
        }
    }

    //handles the teconics on the planets surface
    pub fn tectonics(&mut self,years:f32,sim_info: &mut SimInfo){        
        //for every cell with plate info, move according to plate
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position = glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
            //put cell pos into cell data
            cell.contents.position=cell.position.into();
        }

        //update counter, check if exceeds interval
        if sim_info.triangulation_interval > self.since_triangulation{
           self.since_triangulation +=years;
            return;
        }
        self.since_triangulation = 0.0;
        
        //get edges 
        let edges = indices_to_edges(&self.triangles);

        //filter edges to get only ones on plate boundries, then test for the collisions
        let plate_boundaries:Vec<&(usize,usize)> = edges.iter()
            .filter(|e| 
                &self.cells[e.0].plate != &self.cells[e.1].plate)
                .collect();
            //let plate_boundaries = edges;

        //plate boundery edges currently colliding
        let mut colliding = Vec::with_capacity(plate_boundaries.len());
        //plate boundry edges where plates are diverging
        let mut diverging = Vec::with_capacity(plate_boundaries.len());
        //find and store boundries of each type
        for edge in plate_boundaries{
            //get edge length
            let edge_length = edge_length(&self.cells, edge);
            //if edge length too low, colliding
            if edge_length < self.cell_distance{
                colliding.push(edge);
            }
            //if edge too long, diverging
            else if edge_length > self.cell_distance*2.0{
                diverging.push(edge);
            }
        }
        //now deal with the different types of boundries
        for edge in colliding{
            //sort by density
            let dense_sorted =
            if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                (edge.0,edge.1)
            }else{
                (edge.1,edge.0)
            };
            //when two collide remove the denser one, as it subducts
            self.remove_cell(dense_sorted.0,dense_sorted.1);
            self.cells[dense_sorted.1].contents.height +=0.1;
            //then marks cell as unused by adding to the cell bank    
            self.bank.insert(dense_sorted.0);
        }
        //get index of all cells
        let mut new_cells:Vec<usize> = self.bank.drain().collect();
        //handle divering plate boundaries, place new cells at their mid points when able
        for edge in diverging{
            if let Some(cell) = new_cells.pop(){
                self.add_cell(*edge,cell);
            }
            else{
                break;
            }
        }
        new_cells.into_iter().for_each(|c| _=self.bank.insert(c));
        //now triangulate every point across plate edge
        //get all points on boundry
        /*
        //let boundry_cells = 
        //filter out triangles connected to those points
        let mut surrounding_tris = Vec::with_capacity(16);
            //triangulate every point along plate boundery
            self.triangles = self.triangles.chunks(3)
                .filter(|chunk| {
                    //if triangle contains both parents, record triangle
                    if chunk.contains(&(edge.0 as u32))||chunk.contains(&(edge.1 as u32)){
                        chunk.iter()
                            .for_each(|c| surrounding_tris.push(*c));
                        false
                    }else{
                        true
                    }
                })
                .flatten()//flatten to remove seperation of triangles
                .map(|n|*n)
                .collect();
            let points = self.cells.iter().map(|c|c.position).collect();
            self.triangles.append(&mut flip_triangulate(&points, surrounding_tris)); */
    }
}