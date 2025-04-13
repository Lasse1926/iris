use std::{cmp::Ordering, fmt::Debug};
use std::fmt;
use egui::Vec2;
use image::{Pixel, Rgb};

use super::WINDOW_ID;

#[derive(Debug,PartialEq)]
pub enum ColorSpace {
    Rgb,
    CieLab,
    OkLab,
    XYZ,
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
#[derive(Clone)]
pub struct AvarageRgb {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub color_n:u32,
    pub texture: Option<egui::TextureHandle>,
    pub colors:Vec<AvarageRgb>,
    pub color_info_window_open:bool,
    pub id:usize,
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

        WINDOW_ID.with(|thread_id|{

            let r = rgb.channels()[0];
            let g = rgb.channels()[1];
            let b = rgb.channels()[2];

            let id = thread_id.get();
            thread_id.set(id+1);

            AvarageRgb {
                r,
                g,
                b,
                color_n:1,
                texture:None,
                colors:vec![],
                color_info_window_open:false,
                id,
            }
        })
    }

    pub fn _avarage(&mut self,comp: &AvarageRgb){
        self.color_n += comp.color_n;
        self.r += comp.r/self.color_n as u8;
        self.g += comp.g/self.color_n as u8;
        self.b += comp.b/self.color_n as u8;
        self.colors.append(&mut comp.colors.clone());
    }

    pub fn avarage_with_rgb(&mut self,comp: &Rgb<u8>,color_grad:f32){

        let new_r = comp.channels()[0] as u32;
        let new_g = comp.channels()[1] as u32;
        let new_b = comp.channels()[2] as u32;

        let r = (self.r as u32).pow(2) * self.color_n; 
        let g = (self.g as u32).pow(2) * self.color_n; 
        let b = (self.b as u32).pow(2) * self.color_n; 

        self.r = 254.min(((r + new_r.pow(2))/(self.color_n+1)).isqrt())as u8;
        self.g = 254.min(((g + new_g.pow(2))/(self.color_n+1)).isqrt())as u8;
        self.b = 254.min(((b + new_b.pow(2))/(self.color_n+1)).isqrt())as u8;
        if color_grad > 0.1 {
            let difference = self.colors.contains(&AvarageRgb::from_rgb(*comp));
            if !difference && OkLab::from_rgb(&self.to_rgb()).distance_to_lab(&OkLab::from_rgb(comp)) >= 0.1{
                self.colors.push(AvarageRgb::from_rgb(*comp));
            }
        } 
        self.color_n += 1;

    }
    pub fn color_info_window_show(&mut self,ctx:&egui::Context){
        if self.color_info_window_open {
            let mut window_open = self.color_info_window_open;
            egui::Window::new(format!("{}|{}|{}",self.r,self.g,self.b)).id(egui::Id::new(self.id)).open(&mut window_open).show(ctx, |ui| {
                if let Some(texture) = &self.texture {
                    ui.add(
                        egui::Image::from_texture(texture)
                    );
                }
                let rgb = Rgb::from([self.r,self.g,self.b]);
                ui.label(format!("RGB : {},{},{}",self.r,self.g,self.b));
                let hsl = HSL::from_rgb(&rgb);
                ui.label(format!("HSL : {:.2},{:.2},{:.2}",hsl.h,hsl.s,hsl.l));
                let ok_lab = OkLab::from_rgb(&rgb);
                let cie_lab = CieLab::from_rgb(rgb);
                ui.label(format!("OkLab : {:.2},{:.2},{:.2}",ok_lab.l,ok_lab.a,ok_lab.b));
                ui.label(format!("CieLab : {:.2},{:.2},{:.2}",cie_lab.l,cie_lab.a,cie_lab.b));
                egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                    if self.colors.len() > 0 {
                        egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                            let aw = ui.available_width();
                            egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                                let mut column_count = 0;
                                for c in &self.colors{
                                    if let Some(texture) = &c.texture {
                                        ui.add(
                                            egui::Image::from_texture(texture)
                                        );
                                        column_count += 1;
                                        if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                                            ui.end_row();
                                            column_count = 0;
                                        }
                                    }
                                }
                            });
                        });
                    }
                });
            });
            self.color_info_window_open = window_open;
        }
    }
}

impl fmt::Display for AvarageRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.r,self.g,self.b)
    }
}

impl PartialEq for AvarageRgb {
    fn eq(&self, other: &Self) -> bool {
        let dist = OkLab::from_rgb(&self.to_rgb()).distance_to_lab(&OkLab::from_rgb(&other.to_rgb()));
        dist <= 0.1
    }
}

pub struct XYZ {
    x:f32,
    y:f32,
    z:f32,
}

impl XYZ {
    pub fn from_rgb(rgb:&Rgb<u8>) -> Self {
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

        Self{x,y,z}
    }
}

pub const XYZ_D65:XYZ = XYZ{x:95.047,y:100.0,z:108.883};

pub struct CieLab {
    l:f32,
    a:f32,
    b:f32,
}

impl CieLab {
    pub fn new(l:f32,a:f32,b:f32)-> Self{
        Self{l,a,b}
    } 

    pub fn distance_to_lab_squared(&self,comp:&CieLab) -> f32 {
        (self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)
    }
    pub fn distance_to_lab(&self,comp:&CieLab) -> f32 {
        ((self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)).sqrt()
    }

    pub fn from_xyz(xyz:&XYZ) -> Self{
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
        Self::from_xyz(&XYZ::from_rgb(&rgb))
    }
}

impl fmt::Display for CieLab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.l,self.a,self.b)
    }
}


pub struct OkLab{
    l:f32,
    a:f32,
    b:f32,
}

impl OkLab {
    pub fn new(l:f32,a:f32,b:f32) -> Self {
        OkLab{l,a,b}
    }
    pub fn from_xyz(xyz:&XYZ) -> Self{
        let x = xyz.x;
        let y = xyz.y;
        let z = xyz.z;

        let mut m_1 = 0.8189330101 * x + 0.3618667424 * y - 0.1288597137 * z;
        let mut m_2 = 0.0329845436 * x + 0.9293118715 * y + 0.0361456387 * z;
        let mut m_3 = 0.0482003018 * x + 0.2643662691 * y + 0.6338517070 * z;

        m_1 = m_1.powf(1.0/3.0);
        m_2 = m_2.powf(1.0/3.0);
        m_3 = m_3.powf(1.0/3.0);

        let l = 0.2104542553 * m_1 + 0.7936177850 * m_2 - 0.0040720468 * m_3;
        let a = 1.9779984951 * m_1 - 2.4285922050 * m_2 + 0.4505937099 * m_3;
        let b = 0.0259040371 * m_1 + 0.7827717662 * m_2 - 0.8086757660 * m_3;

        OkLab{l,a,b}
    }
    pub fn from_rgb(rgb:&Rgb<u8>) -> Self {

        let r = rgb.channels()[0] as f32 /255.0;
        let g = rgb.channels()[1] as f32 /255.0;
        let b = rgb.channels()[2] as f32 /255.0;

        let mut l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let mut m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let mut s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
          // Math.crb (cube root) here is the equivalent of the C++ cbrtf function here: https://bottosson.github.io/posts/oklab/#converting-from-linear-srgb-to-oklab
        l = l.powf(1.0/3.0); 
        m = m.powf(1.0/3.0); 
        s = s.powf(1.0/3.0);
        Self{
        l: l * 0.2104542553 + m * 0.7936177850 + s * 0.0040720468,
        a: l * 1.9779984951 + m * -2.4285922050 + s * 0.4505937099,
        b: l * 0.0259040371 + m * 0.7827717662 + s * 0.8086757660
        }
    }
    // pub fn from_rgb(rgb:&Rgb<u8>) -> Self {
    //     Self::from_xyz(&XYZ::from_rgb(&rgb))
    // }
    pub fn distance_to_lab_squared(&self,comp:&OkLab) -> f32 {
        (self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)
    }
    pub fn distance_to_lab(&self,comp:&OkLab) -> f32 {
        ((self.l - comp.l).powf(2.0)+(self.a - comp.a).powf(2.0)+(self.b - comp.b).powf(2.0)).sqrt()
    }
}

impl fmt::Display for OkLab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.l,self.a,self.b)
    }
}

pub struct HSL {
    pub h:f32,
    pub s:f32,
    pub l:f32,
}

#[derive(Debug,PartialEq)]
pub enum HSLDistanceType {
    Hue,
    Saturation,
    Lightness,
}

impl HSL {
    pub fn new(h:f32,s:f32,l:f32) -> Self {
        Self{h,s,l}
    }
    pub fn from_rgb(rgb:&Rgb<u8>) -> Self {
        let r = rgb.channels()[0] as f32 /255.0;
        let g = rgb.channels()[1] as f32 /255.0;
        let b = rgb.channels()[2] as f32 /255.0;

        let v_max = r.max(g.max(b));
        let v_min = r.min(g.min(b));

        let mut h = (v_max+v_min) / 2.0;
        let l = (v_max+v_min) / 2.0;

        if v_max == v_min {
            return Self{h:0.0,s:0.0,l};
        }
        
        let d = v_max - v_min;

        let s = if l> 0.5 {d/(2.0 - v_max - v_min)} else {b/(v_max+v_min)};
        if v_max == r {h = (g-b)/d+(if g<b {6.0} else {0.0})};
        if v_max == g {h = (b-r)/d+2.0};
        if v_max == b {h = (r-g)/d+4.0};

        h = h/6.0;

        Self{h,s,l}
    }
    pub fn hue_distance(&self,hsl:&HSL) -> f32 {
        (self.h.powf(2.0) - hsl.h.powf(2.0)).sqrt() 
    }
    pub fn saturation_distance(&self,hsl:&HSL) -> f32 {
        (self.s.powf(2.0) - hsl.s.powf(2.0)).sqrt() 
    }
    pub fn lightness_distance(&self,hsl:&HSL) -> f32 {
        (self.l.powf(2.0) - hsl.l.powf(2.0)).sqrt() 
    }
    
}

impl fmt::Display for HSL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({},{},{})",self.h,self.s,self.l)
    }
}
