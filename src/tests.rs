
//external crates
use nalgebra_glm as glm;

//internal modules
use crate::planet::utils::{self, *};

//testing the connect_point function, should return 
#[test]
fn connect_point_to_tri(){
    //initial tris the point will be inserted into
    let start_tris:Vec<u32> = vec![0,1,2];
    //use the method
    let result = utils::connect_point(start_tris, 3);
    //expect that the triangles resulting are clockwise 
    let expected = vec![
        0,1,3,
        1,2,3,
        2,0,3
    ];
    //test if the results are correct
    assert_eq!(result,expected);
}

//tests if circumcenter found correctly by comparing with online calculator
#[test]
fn circumcenter_correct_location(){
    let test_points = vec![
        glm::vec3(0.0, 100.0, 0.0),
        glm::vec3(100.0, -100.0, 0.0),
        glm::vec3(-100.0, -100.0, 0.0),

        glm::vec3(-100.0, 0.0, 0.0),
        glm::vec3(0.0, 10.0, 0.0),
        glm::vec3(100.0, 0.0, 0.0)];

    let tri_a = vec![0,1,2];
    let tri_b = vec![3,4,5];
    //results from online calc given same tri
    let expected_a = glm::vec3(0.0,-25.0,0.0);
    let expected_b = glm::vec3(0.0,-495.0,0.0);

    let result_a = utils::circumcenter(&test_points, tri_a);
    let result_b = utils::circumcenter(&test_points, tri_b);

    assert_eq!(result_a,expected_a);
    assert_eq!(result_b,expected_b);
}

//tests if polygon triangulation of y-monotone polygon works correctly
#[test]
fn monotone_poly_triangulation(){
    let test_points = vec![
        glm::vec3(0.0, 10880.0, 0.0),
        glm::vec3(0.0, 10880.0, 0.0),
        glm::vec3(0.0, 100.0, 0.0),
        glm::vec3(-5.0, 80.0, 0.0),
        glm::vec3(-8.0, 0.0, 0.0),
        glm::vec3(-5.0, -70.0, 0.0),
        glm::vec3(0.0, -100.0, 0.0),
        glm::vec3(5.0, -70.0, 0.0),
        glm::vec3(5.0, 0.0, 0.0),
        glm::vec3(5.0, 80.0, 0.0),
        glm::vec3(0.0, 10880.0, 0.0),
        glm::vec3(0.0, 10880.0, 0.0),
        glm::vec3(0.0, 10880.0, 0.0),
        glm::vec3(0.0, 10880.0, 0.0),];

    let polygon = (2..8+2).collect();

    triangulate_monotone(&test_points, polygon);
}