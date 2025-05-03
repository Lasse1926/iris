use std::fmt::Display;
use std::{f32::consts::PI,path::PathBuf};
use egui::ColorImage;
use image::{DynamicImage, GenericImageView, Pixel, RgbaImage};
use image::{ Rgb, RgbImage,ImageReader};

use crate::iris_color::AvarageRgb;

use super::WINDOW_ID;
use super::iris_color;

#[derive(Default)]
pub struct ImageCreator{
    id:usize,
    pub open:bool,
}

impl ImageCreator {
    pub fn new() -> Self {
        WINDOW_ID.with(|thread_id|{

            let id = thread_id.get();
            thread_id.set(id+1);
            
            Self{
                id,
                open:true,
            }
        })
    }
    pub fn show(&mut self,ctx:&egui::Context){
        if self.open {
            let mut window_open = self.open;
            egui::Window::new("ImageCreator").id(egui::Id::new(self.id)).open(&mut window_open).show(ctx,|ui|{
                if ui.add(egui::Button::new("gen")).clicked() {
                    let mut rect = PieColorComp::new(vec![
                        AvarageRgb::from_rgb(Rgb::from([255,0,0]),[0,0]),
                        AvarageRgb::from_rgb(Rgb::from([0,255,0]),[0,0]),
                        // AvarageRgb::from_rgb(Rgb::from([127,255,0])),
                        AvarageRgb::from_rgb(Rgb::from([0,0,255]),[0,0])
                    ],64);
                    rect.generate_pie();
                    rect.save_img();
                }
            });
            self.open = window_open;
        }
    }
}

pub struct RGBRect;

impl RGBRect {
    #[allow(dead_code)]
    pub fn rgb_rect_x(x:f32) -> [f32;3] {
        let r = (-9.0*x.powf(2.0) + 3.0*x + 0.75).max(0.0);
        let g = (-9.0*x.powf(2.0) + 9.0*x - 1.25).max(0.0);
        let b = (-9.0*x.powf(2.0) + 15.0*x - 5.25).max(0.0);

        let b_2 = (-9.0*x.powf(2.0) -3.0*x + 0.75).max(0.0); 
        let r_2 = (-9.0*x.powf(2.0) + 21.0*x - 11.25).max(0.0);

        [r + r_2,g,b + b_2]
    }
    #[allow(dead_code)]
    pub fn generate_image() {
        let mut img = RgbImage::new(64,64);
        for x in 0..64 {
            let rgb = Self::rgb_rect_x(x as f32 / 64.0);  

            let mut rgb_u8:[u8;3]=[0,0,0];

            for y in 0..64 {

                rgb_u8[0] = (rgb[0] * 255.0) as u8;
                rgb_u8[1] = (rgb[2] * 255.0) as u8;
                rgb_u8[2] = (rgb[1] * 255.0) as u8;

                if y > 32 {
                    let step:f32  = 1.0 - ((y as f32 - 32.0)/32.0);
                    rgb_u8[0] = (rgb_u8[0] as f32 * step) as u8;
                    rgb_u8[1] = (rgb_u8[1] as f32 * step) as u8;
                    rgb_u8[2] = (rgb_u8[2] as f32 * step) as u8;
                }
                if y < 32 {
                    let step:f32  = f32::abs((y as f32 - 32.0)/32.0);
                    rgb_u8[0] = (rgb_u8[0] as f32 + (255 - rgb_u8[0])as f32 * step) as u8;
                    rgb_u8[1] = (rgb_u8[1] as f32 + (255 - rgb_u8[1])as f32 * step) as u8;
                    rgb_u8[2] = (rgb_u8[2] as f32 + (255 - rgb_u8[2])as f32 * step) as u8;
                }
                img.put_pixel(x, y, Rgb(rgb_u8));
            }
        }
        let _ = img.save("./created_images/rgb_rect.png");
    }
}
pub trait Draw {
    fn draw_rect(&mut self,parent:&mut HSLRect); 
    fn draw_bar(&mut self,parent:&mut HSLRect); 
}
#[derive(Clone)]
pub struct HSLRect{
    pub img_rect:RgbImage,
    pub img_bar:RgbImage,
    pub size:[u32;2],
    pub obj: Vec<RGBMarker>,
    pub hue:f32,
}

impl HSLRect {
    pub fn new(size:[u32;2],hue:f32) -> Self {
        let img_rect = RgbImage::new(size[0],size[1]);
        let img_bar = RgbImage::new(size[0],size[1]/4);
        HSLRect{size,obj:vec![],hue,img_rect,img_bar}
    }
    pub fn generate_sl_rect(&mut self){
        if self.obj.len() > 0 {
            self.hue = iris_color::HSL::from_rgb(&self.obj[0].rgb).h;
        }
        for x in 0..self.size[0] {
            for y in 0..self.size[1]{
                self.img_rect.put_pixel(x, y,self.pos_to_rgb_rect([x,y]));
            }
        }
        let mut clone_obj = self.obj.clone();
        for m in clone_obj.iter_mut() {
            m.draw_rect(self);
        }
    }
    #[allow(dead_code)]
    pub fn save_rect(&self){ 
        let _ = self.img_rect.save("./created_images/HSL_saturation_lightness_rect.png");
    }
    #[allow(dead_code)]
    pub fn save_bar(&self){ 
        let _ = self.img_bar.save("./created_images/HSL_hue_rect.png");
    }
    pub fn pos_to_rgb_rect(&self,pos:[u32;2]) -> Rgb<u8> {
        let s = pos[0] as f32/self.size[0] as f32;
        let l = pos[1] as f32/self.size[1] as f32;
        let hsl = iris_color::HSL::new(self.hue,s,l);
        hsl.to_rgb()
    }
    pub fn rgb_color_to_position_rect(&self,rgb:&Rgb<u8>) -> [u32;2] {
        let hsl = iris_color::HSL::from_rgb(rgb); 
        let x = hsl.s * self.size[0]as f32;
        let y = hsl.l * self.size[1]as f32;
        [x as u32,y as u32]
    }
    pub fn generate_h_bar(&mut self){
        for x in 0..self.size[0] {
            for y in 0..self.size[1]/4{
                self.img_bar.put_pixel(x, y,self.pos_to_rgb_bar(x as f32));
            }
        }
        let mut clone_obj = self.obj.clone();
        for m in clone_obj.iter_mut() {
            m.draw_bar(self);
        }
    }
    pub fn add_marker(&mut self,new_color:&mut AvarageRgb,size:u32,border_size:u32) -> bool {
        if new_color.marked{
            let new_marker = RGBMarker::new(new_color.to_rgb(),size,border_size);
            self.obj.push(new_marker);
            return true;
        }else{
            return false;
        }
    }
    #[allow(dead_code)]
    pub fn remove_marker(&mut self,new_color:&mut AvarageRgb) -> bool {
        let rgb = new_color.to_rgb();
        let index = self.obj.iter().position(|r| r.rgb == rgb); 
        if let Some(i) = index {
            self.obj.remove(i);
            return true;
        }else{
            return false;
        }
    }
    pub fn pos_to_rgb_bar(&self,x:f32) -> Rgb<u8> {
        let h = (360.0/self.size[0] as f32) * x;
        let hsl = iris_color::HSL::new(h, 1.0,0.5);
        hsl.to_rgb()
    }
    pub fn rgb_color_to_position_bar(&self,rgb:&Rgb<u8>) -> u32{
        let hsl = iris_color::HSL::from_rgb(rgb); 
        let x = hsl.h / (360.0/self.size[0] as f32);
        x as u32
    }
}
#[derive(Clone)]
pub struct RGBMarker {
    rgb:Rgb<u8>,
    size:u32,
    border_size:u32,
}

impl RGBMarker {
    pub fn new(rgb:Rgb<u8>,size:u32,border_size:u32) -> Self{
        Self{rgb,size,border_size} 
    } 
}

impl Draw for RGBMarker{
    fn draw_rect(&mut self,parent: &mut HSLRect) {
        let rgb_pos = parent.rgb_color_to_position_rect(&self.rgb);
        let x_start = if let Some(val) = rgb_pos[0].checked_sub(self.size) {val} else {0};
        let y_start = if let Some(val) = rgb_pos[1].checked_sub(self.size) {val} else {0};
        let x_end = if let Some(val) = rgb_pos[0].checked_add(self.size) {val} else {u32::MAX};
        let y_end = if let Some(val) = rgb_pos[1].checked_add(self.size) {val} else {u32::MAX};
        for x in x_start..x_end{
            for y in y_start..y_end{
                let dist = ((x as f32 - rgb_pos[0]as f32).powf(2.0) + (y as f32 - rgb_pos[1] as f32).powf(2.0)).sqrt();
                if x < parent.size[0] && y < parent.size[1] {
                    if dist < (self.size/2) as f32 {
                        parent.img_rect.put_pixel(x, y,self.rgb);
                    }
                    if dist >= (self.size/2) as f32 && dist <= (self.size/2) as f32 + (self.border_size as f32/2.0) {
                        parent.img_rect.put_pixel(x, y,Rgb::from([255,255,255]));
                    }
                    if dist > (self.size/2) as f32 + (self.border_size as f32/2.0) && dist <= (self.size/2) as f32 + self.border_size as f32 {
                        parent.img_rect.put_pixel(x, y,Rgb::from([0,0,0]));
                    }
                }
            }
        }
    }    
    fn draw_bar(&mut self, parent: &mut HSLRect) {
        let rgb_pos = parent.rgb_color_to_position_bar(&self.rgb);
        let x_start = if let Some(val) = rgb_pos.checked_sub(self.size) {val} else {0};
        let x_end = if let Some(val) = rgb_pos.checked_add(self.size) {val} else {u32::MAX};
        for x in x_start..x_end{
            for y in 0..(parent.size[1]/4){
                let dist = (x as f32 - rgb_pos as f32).abs();
                if x < parent.size[0] && y < parent.size[1]/2 {
                    if dist < (self.size/4) as f32 {
                        parent.img_bar.put_pixel(x, y,self.rgb);
                    }
                    if dist >= (self.size/4) as f32 && dist <= (self.size/4) as f32 + (self.border_size as f32/2.0) {
                        parent.img_bar.put_pixel(x, y,Rgb::from([255,255,255]));
                    }
                    if dist > (self.size/4) as f32 + (self.border_size as f32/2.0) && dist <= (self.size/4) as f32 + self.border_size as f32 {
                        parent.img_bar.put_pixel(x, y,Rgb::from([0,0,0]));
                    }
                    if dist <= ((self.size/4) + self.border_size/2) as f32{
                        if y <= 0 + self.border_size || y >= parent.size[1]/4 -1 - self.border_size{
                            parent.img_bar.put_pixel(x, y,Rgb::from([255,255,255]));
                        }
                        if y <= 0 + self.border_size/2 || y >= parent.size[1]/4 -1 - self.border_size/2{
                            parent.img_bar.put_pixel(x, y,Rgb::from([0,0,0]));
                        }
                    }
                }
            }
        }
    }
}

pub struct PieColorComp {
    pub img:RgbImage,
    pub size:u32,
    pub colors:Vec<AvarageRgb>
}

impl PieColorComp {
    pub fn new(colors:Vec<AvarageRgb>,size:u32) -> Self {
        let img = RgbImage::new(size,size);
        Self {
            img,
            size,
            colors,
        }
    }

    pub fn generate_pie(&mut self){
        for x in 0..self.size{
            for y in 0..self.size{
                let b_length = ((x as f32 - self.size as f32/2.0).powf(2.0) + (y as f32 - self.size as f32/2.0).powf(2.0)).sqrt();
                let mut angle = ((y as f32- self.size as f32/2.0)/b_length).acos();
                if x >= self.size/2 {
                    angle = PI*2.0 - angle;
                }
                let color_angle_step = PI*2.0 / self.colors.len() as f32;
                let vec_to_center = [self.size as f32/2.0 - x as f32,self.size as f32/2.0 - y as f32];
                let dist_to_center = (vec_to_center[0].powf(2.0) + vec_to_center[1].powf(2.0)).sqrt();
                if self.colors.len() > 0 && dist_to_center <= self.size as f32 /2.0 - (self.size/10).min(5) as f32 {
                    let target_color = (angle/color_angle_step).floor() as usize;
                    if target_color < self.colors.len() {
                        self.img.put_pixel(x, y,self.colors[target_color].to_rgb());
                    }else{
                        self.img.put_pixel(x, y,self.colors[0].to_rgb());
                    }
                }else {
                    self.img.put_pixel(x, y,Rgb::from([0,0,0]));
                }

            }
        }
    }

    pub fn save_img(&self){
        let _ = self.img.save("./created_images/pie_color_comp.png");
    }
    
}
#[derive(Default,PartialEq)]
pub enum DisplayOption {
    GrayScale(Option<egui::TextureHandle>), 
    #[default]
    Default,
}

impl Display for DisplayOption{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Default => write!(f,"default"),
            Self::GrayScale(..) => write!(f,"gray_scale"),
        }
    }
}
    
pub struct ImageEditor {
    pub img:RgbaImage,
    pub img_width:u32,
    pub img_hight:u32,
    pub original_img_path:PathBuf,
    pub display_selection:DisplayOption,
    pub image_reader:DynamicImage,
}
impl ImageEditor {
    pub fn new(path:PathBuf) -> Self{
        let image_reader = ImageReader::open(path.clone()).unwrap().decode().unwrap(); 
        let img_hight = image_reader.height();
        let img_width = image_reader.width();
        let img = RgbaImage::new(image_reader.width(),image_reader.height());
        let original_img_path = path;
        let display_selection = DisplayOption::Default;
        Self{
            image_reader,
            img,
            img_width,
            img_hight,
            original_img_path,
            display_selection,
        }
    } 

    pub fn generate_gray_scale_img(&mut self,ui:&mut egui::Ui){
        // self.img = self.image_reader.grayscale().into_rgb8();
        for x in 0..self.img_width {
            for  y in 0..self.img_hight{
                let mut pixel = self.image_reader.get_pixel(x, y);
                let mut gray_scale_rgb = iris_color::HSL::from_rgb(&pixel.to_rgb());
                gray_scale_rgb.s = 0.0;
                pixel.0[0] = gray_scale_rgb.to_rgb().0[0]; 
                pixel.0[1] = gray_scale_rgb.to_rgb().0[1]; 
                pixel.0[2] = gray_scale_rgb.to_rgb().0[2]; 
                self.img.put_pixel(x, y, pixel);
            }
        }
        self.display_selection = DisplayOption::GrayScale(Some(ui.ctx().load_texture("color_text",ColorImage::from_rgba_premultiplied([self.img_width as usize,self.img_hight as usize],&self.img),egui::TextureOptions::NEAREST)));
    }

    pub fn save_img(&self){
        let file_name = self.original_img_path.file_name().unwrap().to_str();
        let _ = self.img.save(format!( "./created_images/{}_{}.png",file_name.unwrap_or("unnamed"),self.display_selection));
    }
}
