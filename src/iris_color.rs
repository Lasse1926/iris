use std::fmt::Debug;
use std::fmt;
use image::{Pixel, Rgb};

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
    pub fn avarage(&mut self,comp: &AvarageRgb){
        self.color_n += comp.color_n;
        self.r += comp.r/self.color_n as u8;
        self.g += comp.g/self.color_n as u8;
        self.b += comp.b/self.color_n as u8;
    }

    pub fn avarage_with_rgb(&mut self,comp: &Rgb<u8>){

        let r = comp.channels()[0];
        let g = comp.channels()[1];
        let b = comp.channels()[2];
        
        self.color_n += 1;

        let _ = self.r.checked_add((r as u32/self.color_n) as u8);
        let _ = self.g.checked_add((g as u32/self.color_n) as u8);
        let _ = self.b.checked_add((b as u32/self.color_n) as u8);
    }
}

impl fmt::Display for AvarageRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.r,self.g,self.b)
    }
}
