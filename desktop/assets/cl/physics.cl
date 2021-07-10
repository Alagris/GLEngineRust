
__kernel void main(__global float * buffer){
    buffer[get_global_id(0)] = 1.53;
}