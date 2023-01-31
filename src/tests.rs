
//external crates
use nalgebra_glm as glm;

//internal modules
use crate::planet::utils;

//testing the connect_point function, should return 
#[test]
fn connect_point_to_tri(){
    let test_points = vec![
        glm::vec3(0.0, 100.0, 0.0),
        glm::vec3(100.0, -100.0, 0.0),
        glm::vec3(-100.0, -100.0, 0.0),

        glm::vec3(0.0, 0.0, 0.0)//point to be added to the tri
        ];
    //initial tris the point will be inserted into
    let start_tris:Vec<u32> = vec![0,1,2];
    //use the method
    let result = utils::connect_point(&test_points, start_tris, 3);
    //expect that the triangles resulting are clockwise 
    let expected = vec![
        0,1,3,
        1,2,3,
        2,0,3
    ];
    //test if the results are correct
    assert_eq!(result,expected);
}

//tests if circumcenter found correctly
#[test]
fn circumcenter(){

}