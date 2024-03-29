#version 330 core

//macros
#define WHITE vec3(1.0,1.0,1.0)
#define BLACK vec3(0.0,0.0,0.0)

#define RED vec3(1.0,0.0,0.0)
#define GREEN vec3(0.0,1.0,0.0)
#define BLUE vec3(0.0,0.0,1.0)

#define YELLOW vec3(1.0,1.0,0.0)
#define CYAN vec3(0.0,1.0,1.0)
#define MAGENTA vec3(1.0,0.0,1.0)

//other colors for topographic map
#define SNOW vec3(1.0,.98,.98)
#define ROCK vec3(.545,.271,.075)
#define SAND vec3(.957,.643,.376)
#define DESERT vec3(1.0,.6,.4)
#define VEG vec3(.133,.545,.133)
#define DARK_GREEN vec3(.0,.392,.0)
#define WATER vec3(.075,.278,.643)


//in from geometry shader
in vec3 v_normal;
in float v_humidity;
in float v_temperature;
in float v_height;
in float v_water;

//out to whatever this renders too
out vec4 color;

//uniforms
uniform vec3 to_light;
uniform int map_mode;

//interpolates between three colors
vec3 three_color(vec3 col_a,vec3 col_b,vec3 col_c,float interpolant){
    if (interpolant <.5){
        return mix(col_a,col_b,interpolant*2); 
    }else{
        return mix(col_b,col_c,(interpolant*2)-1.0); 
    }
}

//interpolates between five colors
vec3 five_color(vec3 col_a,vec3 col_b,vec3 col_c,vec3 col_d,vec3 col_e,float interpolant){
    //use three color func after correcting interpolant appropriately
    if (interpolant <.5){
        return three_color(col_a,col_b,col_c,interpolant*2); 
    }else{
        return three_color(col_c,col_d,col_e,(interpolant*2)-1.0); 
    }
}

//produces natural color of the ground based on the attributes of the cell
vec3 natural_color(float humid,float temp){
    vec3 ground = mix(SAND,DARK_GREEN,humid);
    return mix(SNOW,ground,min(temp*5.0,1.0));
}



void main() {
    //-10km should be -1.0 & 10km should be 1.0
    float norm_height = v_height/10.0;
    //0.0g/m^3 to be 0.0 & 100.0g/m^3 to be 1.0
    float norm_humidity = v_humidity/100.0;
    //-50c should be 0.0 & 50c should be 1.0
    float norm_temperature = (v_temperature/100.0)+.5;

    switch (map_mode){
        //natural
        case 0:
            float brightness = max(dot(to_light,v_normal),0.1);

            if(v_height>0.0){
                color = vec4(natural_color(norm_humidity,norm_temperature)*brightness,1.0);
            }
            else{
                color = vec4(vec3(mix(vec4(0.0,0.02,0.15,1.0),vec4(0.0,0.0,0.10,1.0),abs(norm_height*0.5))* brightness),1.0);
            }
            break;
        //height
        case 1:
            color = vec4(vec3(norm_height),1.0);
            break;
        //temp
        case 2:
            color = vec4(five_color(BLUE,CYAN,GREEN,YELLOW,RED,norm_temperature),1.0);
            break;
        //humidity
        case 3:
            color = vec4(three_color(vec3(1.0,0.647,0.0),vec3(1.0,0.859,0.604),vec3(0.392,0.584,0.929),norm_humidity),1.0);
            break;
        //water coverage
        case 4:
            color = vec4(mix(WHITE,WATER,v_water),1.0);
            break;
        //relief map
        case 5:
            if(norm_height>0.0){
                color = vec4(five_color(VEG,VEG,SAND,ROCK,SNOW,norm_height),1.0);
            }
            else{
                color = vec4(WATER,1.0);
            }
            break;
        //normals
        case 6:
            color = vec4(v_normal,1.0);
            break;
    }
}

