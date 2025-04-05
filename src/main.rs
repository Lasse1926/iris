use std::fmt::Debug;
use std::{collections::HashMap,fmt, path::PathBuf};
use std::cell::Cell;
use eframe::egui;
use egui::{Color32, ColorImage, DroppedFile, Vec2};
use image::{GenericImageView, ImageReader, Pixel, Rgb};

mod iris_color;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}
#[derive(Debug)]
struct ImageWindow {
    id:usize,
    path:PathBuf,
    name:String,
    open:bool,
    color_list:HashMap<u32,iris_color::AvarageRgb>,
    color_percent:HashMap<u32,f32>,
    color_gradation:f32,
    color_dist_type:iris_color::ColorSpace,
}

thread_local!(static WINDOW_ID: Cell<usize> = Cell::new(0));


impl ImageWindow {
    fn new (new_file:DroppedFile)-> Self{
        let path = new_file.path.clone().unwrap();
        let mut name = path.file_name().unwrap().to_owned().to_string_lossy().to_string();
        if path.file_stem().unwrap().to_str().unwrap().to_string().len() >= 10 {
            name = path.file_stem().unwrap().to_str().unwrap().to_string()[0..10].to_string() + "." + &path.extension().unwrap().to_string_lossy()
        }
        WINDOW_ID.with(|thread_id|{
            let id = thread_id.get();
            thread_id.set(id+1);
            ImageWindow{path,name,open:true,color_percent:HashMap::new(),color_list:HashMap::new(),color_gradation:50.0,id,color_dist_type:iris_color::ColorSpace::Rgb}
        })
    }
    fn show (&mut self,ctx:&egui::Context){
        if self.open{
            let mut window_open = self.open;
            egui::Window::new(self.name.clone()).id(egui::Id::new(self.id)).open(&mut window_open).show(ctx, |ui| {

                let string_path = "file://".to_owned() + self.path.to_str().unwrap();
                ui.add(
                    egui::Image::new(string_path)
                ); 
                egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                    egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                        let aw = ui.available_width();
                        egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                            for (num,(_id,c)) in self.color_list.iter().enumerate(){
                                if let Some(texture) = &c.texture {
                                    ui.add(
                                        egui::Image::from_texture(texture)
                                    );
                                    if (num+1)%(aw/ui.available_width()) as usize == 0 {
                                        ui.end_row();
                                    }
                                }
                            }
                        });
                    });
                });
                egui::CollapsingHeader::new("Color Percentages").show(ui,|ui|{
                    egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::TOP).with_cross_justify(true),|ui|{
                            for (id,c) in self.color_list.iter(){
                                if let Some(texture) = &c.texture {
                                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP),|ui|{
                                        ui.add(
                                            egui::Image::from_texture(texture)
                                        );
                                        ui.label(format!("{}|{}|{} |=> {}%",c.r,c.g,c.b,self.color_percent[id]*100.0));
                                    });
                                }
                            }
                        });
                    });
                });
                egui::ComboBox::from_label("Select Color Space for distance")
                    .selected_text(format!("{:?}", self.color_dist_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::Rgb, "RGB");
                        ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::Lab, "Lab");
                    }
                );
                let color_deg_max:f32;
                match self.color_dist_type {
                    iris_color::ColorSpace::Lab => color_deg_max = 200.0,
                    iris_color::ColorSpace::Rgb => color_deg_max = 500.0,
                }
                ui.add(egui::Slider::new(&mut self.color_gradation,0.0 ..= color_deg_max).text("Color Gradation"));
                if ui.add(egui::Button::new("Scan")).clicked(){
                    self.scan_image(ui);
                }

            }); 
            self.open = window_open;
        }
    }    
    fn scan_image(&mut self,ui:&mut egui::Ui){
        let image = ImageReader::open(self.path.clone()).unwrap().decode().unwrap(); 
        let size = image.width() as f32 * image.height() as f32;
        self.color_percent = HashMap::new();
        self.color_list = HashMap::new();
        let mut max_dist = f32::MIN;
        let mut min_dist = f32::MAX;
        for (_x,_y,rgba) in image.pixels(){
            if !(rgba.channels()[3]<= 0){
                let rgb = rgba.to_rgb();
                let mut rgb_already_registered = false;
                for (key,value) in self.color_list.iter_mut(){
                    let dist:f32;
                    match self.color_dist_type{
                        iris_color::ColorSpace::Rgb => dist = iris_color::rgb_distance(value.to_rgb(), rgb),
                        iris_color::ColorSpace::Lab => dist = {
                            let lab_a = iris_color::LabColor::from_rgb(value.to_rgb());
                            let lab_b = iris_color::LabColor::from_rgb(rgb);
                            lab_a.distance_to_Lab(&lab_b)
                        }
                    }
                    max_dist = max_dist.max(dist);
                    min_dist = min_dist.min(dist);
                    if dist < self.color_gradation{
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
                    self.color_percent.insert(self.color_list.len() as u32,1.0/size);
                    self.color_list.insert(self.color_list.len() as u32,iris_color::AvarageRgb::from_rgb(rgb));
                }
            }
        }
        for (_id,c) in self.color_list.iter_mut(){
            c.texture = Some(ui.ctx().load_texture("color_text",ColorImage::new([32,32],Color32::from_rgb(c.r, c.g, c.b)),Default::default()))
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
    
