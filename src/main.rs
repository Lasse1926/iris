use std::{collections::HashMap, fmt::{self, write}, path::PathBuf};
use eframe::egui;
use egui::DroppedFile;
use image::{GenericImageView, ImageReader, Pixel, Rgb};

fn rgb_distance(col_a:Rgb<u8>,col_b:Rgb<u8>) -> f32{
    let r_a = col_a.channels()[0] as f32;
    let g_a = col_a.channels()[1] as f32;
    let b_a = col_a.channels()[2] as f32;

    let r_b = col_b.channels()[0] as f32;
    let g_b = col_b.channels()[1] as f32;
    let b_b = col_b.channels()[2] as f32;

    let dist = (r_b - r_a) + (g_b - g_a) + (b_b - b_a);
    f32::abs(dist)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}
#[derive(Debug)]
struct ImageWindow {
    path:PathBuf,
    name:String,
    open:bool,
    color_list:HashMap<u32,AvarageRgb>,
    color_percent:HashMap<u32,f32>,
    color_gradation:f32,
}
#[derive(Debug)]
struct AvarageRgb {
    r:u8,
    g:u8,
    b:u8,
    color_n:u32,
}

impl AvarageRgb {
    fn to_rgb(&self) -> Rgb<u8>{
        Rgb::from([self.r,self.g,self.b])
    }
    fn from_rgb(rgb:Rgb<u8>) -> Self{

        let r = rgb.channels()[0];
        let g = rgb.channels()[1];
        let b = rgb.channels()[2];

        AvarageRgb {r,g,b,color_n:1}
    }
    fn avarage(&mut self,comp: &AvarageRgb){
        self.color_n += comp.color_n;
        self.r += comp.r/self.color_n as u8;
        self.g += comp.g/self.color_n as u8;
        self.b += comp.b/self.color_n as u8;
    }

    fn avarage_with_rgb(&mut self,comp: &Rgb<u8>){

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

impl ImageWindow {
    fn new (new_file:DroppedFile)-> Self{
        let path = new_file.path.clone().unwrap();
        let mut name = path.file_name().unwrap().to_owned().to_string_lossy().to_string();
        if path.file_stem().unwrap().to_str().unwrap().to_string().len() >= 10 {
            name = path.file_stem().unwrap().to_str().unwrap().to_string()[0..10].to_string() + "." + &path.extension().unwrap().to_string_lossy()
        }
        ImageWindow{path,name,open:true,color_percent:HashMap::new(),color_list:HashMap::new(),color_gradation:50.0}
    }
    fn show (&mut self,ctx:&egui::Context){
        if self.open{
            let mut window_open = self.open;
            egui::Window::new(self.name.clone()).open(&mut window_open).show(ctx, |ui| {
                let string_path = "file://".to_owned() + self.path.to_str().unwrap();
                ui.add(
                    egui::Image::new(string_path)
                ); 
                if ui.add(egui::Button::new("Scan")).clicked(){
                    self.scan_image();
                }

            }); 
            self.open = window_open;
        }
    }    
    fn scan_image(&mut self){
        let image = ImageReader::open(self.path.clone()).unwrap().decode().unwrap(); 
        let size = image.width() as f32 * image.height() as f32;
        self.color_percent = HashMap::new();
        self.color_list = HashMap::new();
        for (_x,_y,rgba) in image.pixels(){
            if !(rgba.channels()[3]<= 0){
                let rgb = rgba.to_rgb();
                let mut rgb_already_registered = false;
                for (key,value) in self.color_list.iter_mut(){
                    if rgb_distance(value.to_rgb(), rgb) < self.color_gradation{
                        value.avarage_with_rgb(&rgb);
                        if let Some(percent) = self.color_percent.get_mut(key){
                            *percent += 1.0/size;
                        }
                        //println!("added + calac new avarage: Added Color :{:?} | old color {:?}",rgb,value);
                        rgb_already_registered = true;
                        break;
                    }
                }
                if !rgb_already_registered{

                    println!("new Color: {:?}",rgb);
                    self.color_percent.insert(self.color_list.len() as u32,1.0/size);
                    self.color_list.insert(self.color_list.len() as u32,AvarageRgb::from_rgb(rgb));
                }
            }
        }
        println!("---------------------------------------------------------");
        for (id,rgb) in self.color_list.iter() {
            println!("{}|[]|{}",rgb,self.color_percent[&id])
        }
    }
}

#[derive(Default)]
struct MyEguiApp {
    image_windows:Vec<ImageWindow>,
}

impl MyEguiApp {
    fn has_image_window(&self,path:PathBuf) -> bool{
        for im in &self.image_windows{
            if path == im.path{
                return true;
            }
        }
        return false;
    }
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut image_window_to_remove:Vec<usize> = vec![];
            for (index,w) in self.image_windows.iter_mut().enumerate(){
                if w.open {
                    w.show(ui.ctx());
                }else{
                   image_window_to_remove.push(index); 
                }
            }
            for index in image_window_to_remove{
                self.image_windows.remove(index);
            }
            if ui.add(egui::Button::new("iw")).clicked(){
                println!("{:?}",self.image_windows);
            }
        }); 
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        
        if raw_input.dropped_files.len() >= 1 {
            for file in raw_input.dropped_files.iter(){
                if !self.has_image_window(file.path.clone().expect("No Path")){
                    self.image_windows.push(ImageWindow::new(file.clone())); 
                }
            }
        }
    }
}
    
