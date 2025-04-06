use std::fmt::Debug;
use std::fmt;
use image::{Pixel, Rgb};

#[derive(Debug,PartialEq)]
pub enum ColorSpace {
    Rgb,
    Lab,
}

pub fn rgb_distance(col_a:Rgb<u8>,col_b:Rgb<u8>) -> f32{
    let r_a = col_a.channels()[0] as f32;
    let g_a = col_a.channels()[1] as f32;
    let b_a = col_a.channels()[2] as f32;

    let r_b = col_b.channels()[0] as f32;
    let g_b = col_b.channels()[1] as f32;
    let b_b = col_b.channels()[2] as f32;

    let dist = f32::sqrt(f32::powf(r_b - r_a,2.0) + f32::powf(g_b - g_a,2.0) + f32::powf(b_b - b_a,2.0));
    dist
}

pub fn rgb_distance_squared(col_a:Rgb<u8>,col_b:Rgb<u8>) -> f32{
    let r_a = col_a.channels()[0] as f32;
    let g_a = col_a.channels()[1] as f32;
    let b_a = col_a.channels()[2] as f32;

    let r_b = col_b.channels()[0] as f32;
    let g_b = col_b.channels()[1] as f32;
    let b_b = col_b.channels()[2] as f32;

    let dist = f32::powf(r_b - r_a,2.0) + f32::powf(g_b - g_a,2.0) + f32::powf(b_b - b_a,2.0);
    dist
}

pub struct AvarageRgb {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub color_n:u32,
    pub texture: Option<egui::TextureHandle>,
}

impl Debug for AvarageRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f,"|r: {}|g: {}|b: {} |=> color_n: {}",self.r,self.g,self.b,self.color_n) 
    }
}

impl AvarageRgb {
    pub fn to_rgb(&self) -> Rgb<u8>{
        Rgb::from([self.r,self.g,self.b])
    }
    pub fn from_rgb(rgb:Rgb<u8>) -> Self{

        let r = rgb.channels()[0];
        let g = rgb.channels()[1];
        let b = rgb.channels()[2];

        AvarageRgb {r,g,b,color_n:1,texture:None}
    }

    pub fn _avarage(&mut self,comp: &AvarageRgb){
        self.color_n += comp.color_n;
        self.r += comp.r/self.color_n as u8;
        self.g += comp.g/self.color_n as u8;
        self.b += comp.b/self.color_n as u8;
    }

    pub fn avarage_with_rgb(&mut self,comp: &Rgb<u8>){

        let new_r = comp.channels()[0] as u32;
        let new_g = comp.channels()[1] as u32;
        let new_b = comp.channels()[2] as u32;

        let r = (self.r as u32).pow(2) * self.color_n; 
        let g = (self.g as u32).pow(2) * self.color_n; 
        let b = (self.b as u32).pow(2) * self.color_n; 

        self.r = 254.min(((r + new_r.pow(2))/(self.color_n+1)).isqrt())as u8;
        self.g = 254.min(((g + new_g.pow(2))/(self.color_n+1)).isqrt())as u8;
        self.b = 254.min(((b + new_b.pow(2))/(self.color_n+1)).isqrt())as u8;

        self.color_n += 1;

    }
}

impl fmt::Display for AvarageRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.r,self.g,self.b)
    }
}

pub struct Vec3 {
    x:f32,
    y:f32,
    z:f32,
}

pub const XYZ_D65:Vec3 = Vec3{x:95.047,y:100.0,z:108.883};

pub struct LabColor {
    l:f32,
    a:f32,
    b:f32,
    x:f32,
    y:f32,
    z:f32,
}

impl LabColor {
    pub fn new(l:f32,a:f32,b:f32)-> Self{
        Self{l,a,b,x:XYZ_D65.x,y:XYZ_D65.y,z:XYZ_D65.z}
    } 

    pub fn distance_to_lab_squared(&self,comp:&LabColor) -> f32 {
        (self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)
    }
    pub fn distance_to_lab(&self,comp:&LabColor) -> f32 {
        ((self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)).sqrt()
    }

    pub fn from_xyz(xyz:&Vec3) -> Self{
        let mut var_x = xyz.x/XYZ_D65.x;
        let mut var_y = xyz.y/XYZ_D65.y;
        let mut var_z = xyz.z/XYZ_D65.z;

        var_x = if var_x > 0.008856 {var_x.powf(1.0/3.0)} else {(7.787 * var_x)+(16.0/116.0)};
        var_y = if var_y > 0.008856 {var_y.powf(1.0/3.0)} else {(7.787 * var_y)+(16.0/116.0)};
        var_z = if var_z > 0.008856 {var_z.powf(1.0/3.0)} else {(7.787 * var_z)+(16.0/116.0)};

        let cie_l = (116.0 * var_y) - 16.0;
        let cie_a = 500.0 *(var_x - var_y);
        let cie_b = 200.0 *(var_y - var_z);

        Self::new(cie_l,cie_a,cie_b)
    }
    pub fn from_rgb(rgb:Rgb<u8>)-> Self{
        Self::from_xyz(&rgb_to_xyz(&rgb))
    }
}

impl fmt::Display for LabColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.l,self.a,self.b)
    }
}

pub fn rgb_to_xyz(rgb:&Rgb<u8>) -> Vec3 {
    let mut r = rgb.channels()[0] as f32 /255.0;
    let mut g = rgb.channels()[1] as f32 /255.0;
    let mut b = rgb.channels()[2] as f32 /255.0;

    r = if r > 0.04045 {((r + 0.055)/1.055).powf(2.4)} else {r/12.92};
    g = if g > 0.04045 {((g + 0.055)/1.055).powf(2.4)} else {g/12.92};
    b = if b > 0.04045 {((b + 0.055)/1.055).powf(2.4)} else {b/12.92};

    r = r*100.0;
    g = g*100.0;
    b = b*100.0;

    let x = r * 0.4124 + g * 0.3576 + b * 0.1805;
    let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
    let z = r * 0.0193 + g * 0.1192 + b * 0.9505;

    Vec3{x,y,z}
}
