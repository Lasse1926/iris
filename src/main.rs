use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Debug;
use std::{collections::HashMap,path::PathBuf};
use std::cell::Cell;
use eframe::egui;
use egui::{widgets, Color32, ColorImage, DroppedFile, Vec2};
use image::{GenericImageView, ImageReader, Pixel, Rgb};
use itertools::Itertools;

mod iris_color;
mod iris_image_creation;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}
struct ImageWindow {
    id:usize,
    path:PathBuf,
    main_img_size:[u32;2],
    name:String,
    open:bool,
    color_list:HashMap<u32,iris_color::AvarageRgb>,
    color_percent:HashMap<u32,f32>,
    color_pixel_count:HashMap<u32,u32>,
    color_gradation:f32,
    color_dist_type:iris_color::ColorSpace,
    color_display_threshhold:f32,

    compare_state:CompareState,

    avaraging_system:AvarageingSystem,

    img: Option<iris_image_creation::HSLRect>,
    img_rect:Option<egui::TextureHandle>,
    img_bar:Option<egui::TextureHandle>,
    img_dispaly_generated:bool,
    reload_hsl_rect:bool,
    reload_hsl_bar:bool,
    clean_up_value:f32,
    mark_every_color:bool,
    median_cut_amount:u32,

    avarage_saturation:f32,
    saturation_range:[f32;2],

    avarage_lightness:f32,
    lightness_range:[f32;2],
}

#[derive(Debug,PartialEq)]
enum CompareState {
    Percentages,
    Saturation,
}

#[derive(Debug,PartialEq)]
enum AvarageingSystem {
    DeltaE,
    MedianColor,
    MedianCuttin,
}

#[derive(Clone)]
struct MedianCut {
    median_color:[u8;3],
    colors:Vec<[u8;3]>,
}
thread_local!(static WINDOW_ID: Cell<usize> = Cell::new(0));


impl ImageWindow {
    fn new (new_file:DroppedFile)-> Self{
        let path = new_file.path.clone().unwrap();
        let mut name = path.file_name().unwrap().to_owned().to_string_lossy().to_string();
        if path.file_stem().unwrap().to_str().unwrap().to_string().len() >= 10 {
            name = path.file_stem().unwrap().to_str().unwrap().to_string()[0..10].to_string() + "." + &path.extension().unwrap().to_string_lossy()
        }
        let image = ImageReader::open(path.clone()).unwrap().decode().unwrap(); 
        
        WINDOW_ID.with(|thread_id|{
            let id = thread_id.get();
            thread_id.set(id+1);
            let open = true;
            let color_percent = HashMap::new();
            let color_list = HashMap::new();
            let color_pixel_count = HashMap::new();
            let color_gradation = 50.0;
            let color_dist_type = iris_color::ColorSpace::OkLab;
            let color_display_threshhold = 0.01;
            let compare_state = CompareState::Percentages;
            let avaraging_system = AvarageingSystem::DeltaE;
            let clean_up_value = 0.01;

            let avarage_saturation = 0.0;
            let saturation_range = [0.0,0.0];

            let avarage_lightness = 0.0;
            let lightness_range = [0.0,0.0];

            let main_img_size = [image.width(),image.height()];
            ImageWindow{
                path,
                name,
                open,
                color_percent,
                color_list,
                color_pixel_count,
                color_gradation,
                id,
                color_dist_type,
                color_display_threshhold,
                compare_state,
                avaraging_system,
                img: None,
                img_bar: None,
                img_rect: None,
                img_dispaly_generated: false,
                reload_hsl_rect:false,
                reload_hsl_bar:false,
                clean_up_value,
                mark_every_color:false,
                median_cut_amount:0,
                main_img_size,
                avarage_saturation,
                saturation_range,
                avarage_lightness,
                lightness_range,
            }

        })
    }
    fn generate_color_display(&mut self) {
        if self.color_list.len() <= 0 {
            return;
        }
        let mut color_sorted:Vec<_> = self.color_list.iter_mut().collect();
        color_sorted.sort_by(|a,b| {
            if self.color_percent[a.0] < self.color_percent[b.0] {
                return Ordering::Greater;
            }else{
                return Ordering::Less;
            }
        });
        let mut dom_color:Option<f32> = None; 
        for c in &color_sorted {
            if iris_color::HSL::from_rgb(&c.1.to_rgb()).l >= 0.2 {
                dom_color = Some(iris_color::HSL::from_rgb(&c.1.to_rgb()).h);
                break;
            }        
        } 
        if dom_color.is_none() {
            dom_color = Some(iris_color::HSL::from_rgb(&color_sorted[1].1.to_rgb()).h);
        }
        if self.img.is_none() {
            self.img = Some(iris_image_creation::HSLRect::new([256,128],dom_color.unwrap()));
        }
        if let Some(img) = &mut self.img {
            img.obj.clear();
            for (id,c) in color_sorted{
                if self.color_percent[id] >= self.color_display_threshhold {
                    img.add_marker(c,5,2);
                }
            }
            img.generate_h_bar();
            self.reload_hsl_bar = true;
            img.generate_sl_rect();
            self.reload_hsl_rect = true;
        }
        self.img_dispaly_generated = true;
    }
    fn show (&mut self,ctx:&egui::Context){
        if self.open{
            if (self.img_bar.is_none()|| self.reload_hsl_bar) && self.img_dispaly_generated  {
                if let Some(img) = &self.img {
                    self.img_bar = Some(ctx.load_texture("img_bar",ColorImage::from_rgb([img.size[0].try_into().unwrap(),(img.size[1]/4).try_into().unwrap()],&img.img_bar),Default::default()));
                    self.reload_hsl_bar = false;
                }
            }
            if (self.img_rect.is_none()|| self.reload_hsl_rect) && self.img_dispaly_generated{
                if let Some(img) = &self.img {
                    self.img_rect = Some(ctx.load_texture("img_rect",ColorImage::from_rgb([img.size[0].try_into().unwrap(),img.size[1].try_into().unwrap()],&img.img_rect),Default::default()));
                    self.reload_hsl_rect = false;
                }
            }
            let mut window_open = self.open;
            egui::Window::new(self.name.clone()).id(egui::Id::new(self.id)).open(&mut window_open).show(ctx, |ui| {

                let string_path = "file://".to_owned() + self.path.to_str().unwrap();
                ui.add(
                    egui::Image::new(string_path).shrink_to_fit()
                ); 
                egui::ComboBox::from_label("Select Avaraging Technique")
                    .selected_text(format!("{:?}",self.avaraging_system))
                    .show_ui(ui,|ui|{
                        ui.selectable_value(&mut self.avaraging_system,AvarageingSystem::MedianCuttin,"Median Cutting");
                        ui.selectable_value(&mut self.avaraging_system,AvarageingSystem::DeltaE,"Delta E");
                        ui.selectable_value(&mut self.avaraging_system,AvarageingSystem::MedianColor,"Median Color");
                    });
                ui.separator();
                match self.avaraging_system {

                    AvarageingSystem::DeltaE => {
                        egui::ComboBox::from_label("Select Color Space for distance")
                            .selected_text(format!("{:?}", self.color_dist_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::Rgb, "RGB");
                                ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::CieLab, "CieLab");
                                ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::OkLab, "OkLab");
                            }
                        );
                        let color_deg_max:f32;
                        match self.color_dist_type {
                            iris_color::ColorSpace::CieLab => color_deg_max = 300.0,
                            iris_color::ColorSpace::OkLab => color_deg_max = 2.0,
                            iris_color::ColorSpace::Rgb => color_deg_max = 500.0,
                            _=> color_deg_max = 0.0,
                        }
                        ui.add(egui::Slider::new(&mut self.color_gradation,0.0 ..= color_deg_max).text("Color Gradation"));
                        ui.add(egui::Slider::new(&mut self.clean_up_value,0.0 ..= 0.1).text("Clean up Threshold"))
                            .on_hover_text("Minimum Color distance in OKLab, at which colors get merged after scan. \n (to clean up Duplicate Colors)");
                        if ui.add(egui::Button::new("Scan")).clicked(){
                            self.scan_image_delta_e(ui);
                            self.get_img_data();
                        }
                    }
                    AvarageingSystem::MedianColor => {
                        if ui.button("Scan for Median Color").clicked(){
                            self.scan_image_median_color(ui);
                            self.get_img_data();
                        }
                    },
                    AvarageingSystem::MedianCuttin => {
                        ui.add(egui::Slider::new(&mut self.median_cut_amount,0 ..= 100).text("Median Cut amount")).on_hover_text("n Cuts result in n+1 colors");
                        if ui.button("Scan").clicked(){
                            self.scan_image_median_cutting(ui);
                            self.get_img_data();
                        }
                    },
                }
                ui.separator();
                egui::ComboBox::from_label("Sorted by")
                    .selected_text(format!("{:?}", self.compare_state))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.compare_state,CompareState::Percentages , "Percentages");
                        ui.selectable_value(&mut self.compare_state,CompareState::Saturation , "Saturation");
                    }
                );
                if ui.checkbox(&mut self.mark_every_color,"Select every color").clicked(){
                    for (_id,c) in &mut self.color_list{
                       c.marked = self.mark_every_color; 
                    }
                };
                match self.compare_state {
                    CompareState::Percentages => {  // ----------PERCENTAGE GUI
                        let mut color_sorted:Vec<_> = self.color_list.iter_mut().collect();
                        color_sorted.sort_by(|a,b| {
                            if self.color_percent[a.0] < self.color_percent[b.0] {
                                return Ordering::Greater;
                            }else{
                                return Ordering::Less;
                            }
                        });
                        ui.add(egui::Slider::new(&mut self.color_display_threshhold,0.0 ..= 1.0).text("Color Display Threshold"));
                        egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                                let aw = ui.available_width();
                                egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                                    let mut column_count = 0;
                                    for (id,c) in color_sorted.iter_mut(){
                                        if self.color_percent[id] >= self.color_display_threshhold{
                                            iris_color::color_display(ui, c);
                                            column_count += 1;
                                            if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                                                ui.end_row();
                                                column_count = 0;
                                            }
                                        }
                                    }
                                });
                            });
                        });
                        egui::CollapsingHeader::new("Color Percentages").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::TOP).with_cross_justify(true),|ui|{
                                    for (id,c) in color_sorted.iter_mut(){
                                        if self.color_percent[id] >= self.color_display_threshhold || self.color_display_threshhold <= 0.0{
                                            iris_color::color_display_percent(ui, c,self.color_percent[id].clone());
                                        }
                                    }
                                });
                            });
                        });
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT),|ui|{
                            if let Some(rect) = &self.img_rect {
                                ui.add(
                                    egui::Image::from_texture(rect)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if let Some(bar) = &self.img_bar {
                                ui.add(
                                    egui::Image::from_texture(bar)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if ui.button("Generate").clicked() {
                                self.generate_color_display();
                            }
                        });
                    },
                    CompareState::Saturation => {
                        ui.add(egui::Slider::new(&mut self.color_display_threshhold,0.0 ..= 1.0).text("Color Display Threshold"));
                        let mut color_sorted:Vec<_> = self.color_list.iter_mut().collect();
                        color_sorted.sort_by(|a,b| {
                            if iris_color::HSL::from_rgb(&a.1.to_rgb()).s < (iris_color::HSL::from_rgb(&b.1.to_rgb()).s) {
                                return Ordering::Greater;
                            }else{
                                return Ordering::Less;
                            }
                        });
                        egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                                let aw = ui.available_width();
                                egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                                    let mut column_count = 0;
                                    for (id,c) in color_sorted.iter_mut(){
                                        if self.color_percent[id] >= self.color_display_threshhold{
                                            iris_color::color_display(ui,c);
                                            column_count += 1;
                                            if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                                                ui.end_row();
                                                column_count = 0;
                                            }
                                        }
                                    }
                                });
                            });
                        });
                        egui::CollapsingHeader::new("Color Percentages").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::TOP).with_cross_justify(true),|ui|{
                                    for (id,c) in &mut color_sorted{
                                        if self.color_percent[id] >= self.color_display_threshhold || self.color_display_threshhold == 0.0{
                                            iris_color::color_display_percent(ui,c,self.color_percent[id]);
                                        }
                                    }
                                });
                            });
                        });
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT),|ui|{
                            if let Some(rect) = &self.img_rect {
                                ui.add(
                                    egui::Image::from_texture(rect)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if let Some(bar) = &self.img_bar {
                                ui.add(
                                    egui::Image::from_texture(bar)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if ui.button("Generate").clicked() {
                                self.generate_color_display();
                            }
                        });
                    }
                }
                egui::CollapsingHeader::new("Properties").show(ui,|ui|{
                    ui.label(format!("Size: {}x{}",self.main_img_size[0],self.main_img_size[1]));
                    ui.label(format!("Avarage Saturation: {:.2}%",self.avarage_saturation * 100.0));
                    ui.label(format!("Saturation Range:\n\tMax: {:.2}%\n\tMin: {:.2}%",self.saturation_range[0] * 100.0,self.saturation_range[1] * 100.0));
                    ui.label(format!("Avarage Lightness: {:.2}%",self.avarage_lightness * 100.0));
                    ui.label(format!("Lightness Range:\n\tMax: {:.2}%\n\tMin: {:.2}%",self.lightness_range[0] * 100.0,self.lightness_range[1] * 100.0));
                });
                for (_,color) in self.color_list.iter_mut(){
                    if color.color_info_window_open {
                        color.color_info_window_show(ui.ctx());
                    }
                }
            }); 
            self.open = window_open;
        }
    }    
    fn scan_image_median_color(&mut self,ui:&mut egui::Ui){

        self.color_percent = HashMap::new();
        self.color_list = HashMap::new();

        let mut color_vec:Vec<Rgb<u8>> = vec![];

        let image = ImageReader::open(self.path.clone()).unwrap().decode().unwrap(); 
        let size = image.width() as f64 * image.height() as f64;

        for (_x,_y,rgba) in image.pixels(){
            if !(rgba.channels()[3]<= 0){
                let rgb = rgba.to_rgb();
                color_vec.push(rgb);
            }
        }
         
        color_vec.sort_by(|a,b| a.0[0].partial_cmp(&b.0[0]).unwrap());
        let r:u8; 
        if color_vec.len() % 2 == 0 {
            let upper = color_vec[color_vec.len()/2].0[0];
            let lower = color_vec[(color_vec.len()/2)-1].0[0];

            r = ((upper as u32 + lower as u32)/2).min(255) as u8;
        }else{
            r = color_vec[(color_vec.len() as f32/2.0).ceil() as usize].0[0];
        }
        let g:u8; 
        if color_vec.len() % 2 == 0 {
            let upper = color_vec[color_vec.len()/2].0[1];
            let lower = color_vec[(color_vec.len()/2)-1].0[1];

            g = ((upper as u32 + lower as u32)/2).min(255) as u8;
        }else{
            g = color_vec[(color_vec.len() as f32/2.0).ceil() as usize].0[1];
        }
        let b:u8; 
        if color_vec.len() % 2 == 0 {
            let upper = color_vec[color_vec.len()/2].0[2];
            let lower = color_vec[(color_vec.len()/2)-1].0[2];

            b = ((upper as u32 + lower as u32)/2).min(255) as u8;
        }else{
            b = color_vec[(color_vec.len() as f32/2.0).ceil() as usize].0[2];
        }

        let median_color:Rgb<u8> = Rgb::from([r,g,b]);
        let mut avarage_median = iris_color::AvarageRgb::from_rgb(median_color);
        avarage_median.generate_texture(ui);

        self.color_list.insert(0,avarage_median);
        self.color_percent.insert(0,1.0);
        self.color_pixel_count.insert(0,size as u32);


    }
    fn get_median_color(&self,colors:&mut Vec<[u8;3]>) -> [u8;3] {
        colors.sort_by(|a,b| a[0].partial_cmp(&b[0]).unwrap());
        let r:u8; 
        if colors.len() == 1 {
            return colors[0];
        }
        if colors.len() % 2 == 0 {
            let upper = colors[colors.len()/2][0];
            let lower = colors[(colors.len()/2)-1][0];

            r = ((upper as u32 + lower as u32)/2).min(255) as u8;
        }else{
            r = colors[(colors.len() as f32/2.0).ceil() as usize][0];
        }
        let g:u8; 
        if colors.len() % 2 == 0 {
            let upper = colors[colors.len()/2][1];
            let lower = colors[(colors.len()/2)-1][1];

            g = ((upper as u32 + lower as u32)/2).min(255) as u8;
        }else{
            g = colors[(colors.len() as f32/2.0).ceil() as usize][1];
        }
        let b:u8; 
        if colors.len() % 2 == 0 {
            let upper = colors[colors.len()/2][2];
            let lower = colors[(colors.len()/2)-1][2];

            b = ((upper as u32 + lower as u32)/2).min(255) as u8;
        }else{
            b = colors[(colors.len() as f32/2.0).ceil() as usize][2];
        }

        [r,g,b]
    }

    fn scan_image_median_cutting(&mut self, ui:&mut egui::Ui){
        self.color_percent = HashMap::new();
        self.color_list = HashMap::new();
        self.color_pixel_count = HashMap::new();
        let image = ImageReader::open(self.path.clone()).unwrap().decode().unwrap(); 
        let _size = image.width() as f64 * image.height() as f64;

        let mut color_rgb_values:HashSet<[u8;3]>= HashSet::new();
       

        for (_x,_y,rgba) in image.pixels(){
            if !(rgba.channels()[3]<= 0){
                let rgb = rgba.to_rgb();


                if !color_rgb_values.contains(&rgb.0) {
                    color_rgb_values.insert(rgb.0); 
                }
            }
        }
        let mut color_vec = color_rgb_values.into_iter().collect_vec();
        let all_color_size = color_vec.len();
        let mut cuts:Vec<MedianCut> = vec![MedianCut{median_color:self.get_median_color(&mut color_vec),colors:color_vec}];
        for _ in 0..self.median_cut_amount {
            let target = cuts.pop(); 
            if let Some(mut t) = target {
                let median_cut_pair = self.median_cut(&mut t.colors);
                cuts.push(median_cut_pair[0].clone());
                cuts.push(median_cut_pair[1].clone());
            }
            cuts.sort_by(|a,b| a.colors.len().partial_cmp(&b.colors.len()).unwrap());
        }
        for median_cut in cuts {
            let mut avarage_median = iris_color::AvarageRgb::from_rgb(Rgb::from(median_cut.median_color));
            for c in median_cut.colors.clone().into_iter() {
                if c == median_cut.median_color {
                    break;
                }
                let mut ac_buffer = iris_color::AvarageRgb::from_rgb(Rgb::from(c));
                ac_buffer.generate_texture(ui);
                avarage_median.colors.push(ac_buffer);
            }
            avarage_median.generate_texture(ui);
            let key = self.color_list.len() as u32;
            self.color_list.insert(key,avarage_median);
            self.color_percent.insert(key,median_cut.colors.len() as f32/all_color_size as f32);
        }

    }


    fn median_cut(&self,colors:&mut Vec<[u8;3]>) -> [MedianCut;2] {
        // range = [max,min]
        let mut r_range:[u8;2] = [0,u8::MAX];
        let mut g_range:[u8;2] = [0,u8::MAX];
        let mut b_range:[u8;2] = [0,u8::MAX];

        for c in colors.iter() {
            r_range[0] = r_range[0].max(c[0]);
            r_range[1] = r_range[1].min(c[0]);

            g_range[0] = g_range[0].max(c[1]);
            g_range[1] = g_range[1].min(c[1]);

            b_range[0] = b_range[0].max(c[2]);
            b_range[1] = b_range[1].min(c[2]);
        }
        let biggest_range:usize;

        let r_range_num = r_range[0] - r_range[1];
        let g_range_num = g_range[0] - g_range[1];
        let b_range_num = b_range[0] - b_range[1];

        if  r_range_num > g_range_num && r_range_num > b_range_num {
            biggest_range = 0;
        }else if g_range_num > r_range_num && g_range_num > b_range_num {
            biggest_range = 1;
        }else {
            biggest_range = 2;
        }
        if !biggest_range < 3 {
            panic!();
        }
        colors.sort_by(|a,b| a[biggest_range].partial_cmp(&b[biggest_range]).unwrap());
        let median = colors.len()/2;
        let mut top_slice = colors[0..median].to_vec();
        let mut bot_slice = colors[median..colors.len()].to_vec();
        let top_color = self.get_median_color(&mut top_slice);
        let bot_color = self.get_median_color(&mut bot_slice);
        [MedianCut{colors:top_slice,median_color:top_color},MedianCut{colors:bot_slice,median_color:bot_color}]
    }

    fn scan_image_delta_e(&mut self,ui:&mut egui::Ui){
        let image = ImageReader::open(self.path.clone()).unwrap().decode().unwrap(); 
        let size = image.width() as f64 * image.height() as f64;
        self.color_percent = HashMap::new();
        self.color_list = HashMap::new();
        self.color_pixel_count = HashMap::new();
        let mut max_dist = f32::MIN;
        let mut min_dist = f32::MAX;
        let mut transparent_pixels:f64 = 0.0;
        for (_x,_y,rgba) in image.pixels(){
            if !(rgba.channels()[3]<= 0){
                let rgb = rgba.to_rgb();
                let mut rgb_already_registered = false;
                let mut closest_color_dist:f32 = f32::MAX;
                let mut closest_color_key:Option<u32> = None;
                if self.color_gradation >= 0.0 {
                    for (key,value) in self.color_list.iter_mut(){
                        let dist:f32;
                        match self.color_dist_type{
                            iris_color::ColorSpace::Rgb => dist = iris_color::rgb_distance(value.to_rgb(), rgb),
                            iris_color::ColorSpace::CieLab => dist = {
                                let lab_a = iris_color::CieLab::from_rgb(value.to_rgb());
                                let lab_b = iris_color::CieLab::from_rgb(rgb);
                                lab_a.distance_to_lab(&lab_b)
                            },
                            iris_color::ColorSpace::XYZ => dist = 0.0,
                            iris_color::ColorSpace::OkLab => dist = {
                                let lab_a = iris_color::OkLab::from_rgb(&value.to_rgb());
                                let lab_b = iris_color::OkLab::from_rgb(&rgb);
                                lab_a.distance_to_lab(&lab_b)
                            },
                        }
                        max_dist = max_dist.max(dist);
                        min_dist = min_dist.min(dist);
                        if dist <= self.color_gradation{
                            if closest_color_dist > dist {
                                closest_color_dist = dist;
                                closest_color_key = Some(*key);
                            }
                            rgb_already_registered = true;
                        }
                    }
                }
                if !rgb_already_registered {
                    self.color_percent.insert(self.color_list.len() as u32,(1.0/size)as f32);
                    self.color_pixel_count.insert(self.color_list.len() as u32, 1);
                    self.color_list.insert(self.color_list.len() as u32,iris_color::AvarageRgb::from_rgb(rgb));
                }else if let Some(cck) = closest_color_key{
                    if let Some(value) = self.color_list.get_mut(&cck){
                        if self.color_gradation > 0.0 {
                            value.avarage_with_rgb(&rgb,self.color_gradation);
                        }
                        if let Some(percent) = self.color_percent.get_mut(&cck){
                            *percent += (1.0/size) as f32;
                        }
                        if let Some(count) = self.color_pixel_count.get_mut(&cck){
                            *count += 1;
                        }
                    }
                }
            }else{
                transparent_pixels += 1.0;
            }
        }
        for (_,p) in self.color_percent.iter_mut(){
            *p = ((*p as f64 *size)/(size-transparent_pixels)) as f32;
        }

        for (_id,c) in self.color_list.iter_mut(){
            c.generate_texture(ui);
            for sub_c in c.colors.iter_mut() {
                sub_c.generate_texture(ui);
            }
        }
        self.clean_up();
    }
    fn clean_up(&mut self) {
        let mut id_remove = vec![];
        let id_list = self.color_list.clone();
        for ids in id_list.keys().into_iter().combinations(2){
            if !(id_remove.contains(&ids[0]) || id_remove.contains(&ids[1])){
                if iris_color::OkLab::from_rgb(&self.color_list[ids[0]].to_rgb()).distance_to_lab(&iris_color::OkLab::from_rgb(&self.color_list[ids[1]].to_rgb())) <= self.clean_up_value{
                    let other_value = self.color_list[ids[1]].clone();
                    let other_percent = self.color_percent[ids[1]].clone();
                    let other_pixel = self.color_pixel_count[ids[1]].clone();
                    if let Some(value) = self.color_list.get_mut(ids[0]){
                        value._avarage(&other_value);
                    }
                    if let Some(value) = self.color_percent.get_mut(ids[0]){
                        *value += other_percent;
                    }
                    if let Some(value) = self.color_pixel_count.get_mut(ids[0]){
                        *value = value.checked_add(other_pixel).unwrap_or(u32::MAX);
                    }
                    id_remove.push(ids[1]);
                } 
            }
        }
        for id in id_remove {
            self.color_list.remove(id);
            self.color_percent.remove(id);
            self.color_pixel_count.remove(id);
        }
    }
    fn get_img_data(&mut self){
        let mut avarage_sat:f32 = 0.0;
        let mut max_sat:f32 = 0.0;
        let mut min_sat:f32 = f32::MAX;

        let mut avarage_light:f32 = 0.0;
        let mut max_light:f32 = 0.0;
        let mut min_light:f32 = f32::MAX;
        for (_,c) in self.color_list.iter() {
            let col_sat = iris_color::HSL::from_rgb(&c.to_rgb()).s;
            let col_light = iris_color::HSL::from_rgb(&c.to_rgb()).l;
            avarage_sat += col_sat;
            max_sat = max_sat.max(col_sat);
            min_sat = min_sat.min(col_sat);

            avarage_light += col_light;
            max_light = max_light.max(col_light);
            min_light = min_light.min(col_light);
        }
        avarage_sat = avarage_sat/self.color_list.len() as f32;
        self.avarage_saturation = avarage_sat;
        self.saturation_range = [max_sat,min_sat];
        self.avarage_lightness = avarage_light/self.color_list.len() as f32;
        self.lightness_range = [max_light,min_light]
    }
}

#[derive(Default)]
struct MyEguiApp {
    image_windows:Vec<ImageWindow>,
    image_creation_windows:Vec<iris_image_creation::ImageCreator>,
    color_to_add:[f32;3],
    global_colors:Vec<iris_color::AvarageRgb>,
    compare_window:Vec<ColorCompareWindow>,
    mark_every_color:bool,
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
    fn get_global_selected_colors(&mut self){
        for iw in self.image_windows.iter() {
            for (_,c) in iw.color_list.iter(){
                if c.marked {
                    if !self.global_colors.contains(c){
                        self.global_colors.push(c.clone());
                    }
                    for sub_c in c.colors.iter() {
                        if sub_c.marked && !self.global_colors.contains(sub_c){
                            self.global_colors.push(sub_c.clone());
                        }
                    }
                }else{
                    for sub_c in c.colors.iter() {
                        if sub_c.marked && !self.global_colors.contains(sub_c){
                            self.global_colors.push(sub_c.clone());
                        }
                    }
                }
            }
        }
    }
    fn get_selected_colors(&self)->Vec<iris_color::AvarageRgb>{
        let mut color_to_return:Vec<iris_color::AvarageRgb> = vec![]; 
        for c in self.global_colors.iter() {
            if c.marked {
                color_to_return.push(c.clone());
            }
            for sub_c in c.colors.iter() {
                if sub_c.marked && !self.global_colors.contains(sub_c){
                    color_to_return.push(sub_c.clone());
                }
            }
        }
        color_to_return
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("ToolBar").show(ctx, |ui| {
            if ui.add(egui::Button::new("Image Creation")).clicked(){
                self.image_creation_windows.push(iris_image_creation::ImageCreator::new());
            }
        });
        egui::SidePanel::left("ColorPanle").show(ctx,|ui| {
            if ui.button("Get Colors").on_hover_text("Copies every selected Color into Your Color Palet").clicked(){
                self.get_global_selected_colors();
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP),|ui| {
                ui.color_edit_button_rgb(&mut self.color_to_add);
                if ui.button("+").on_hover_text("Add displayed Color to your color palette").clicked(){
                    let r = (self.color_to_add[0] * 255.0).min(255.0) as u8; 
                    let g = (self.color_to_add[1] * 255.0).min(255.0) as u8; 
                    let b = (self.color_to_add[2] * 255.0).min(255.0) as u8; 
                    let mut color = iris_color::AvarageRgb::from_rgb(Rgb::from([r,g,b]));
                    color.generate_texture(ui);

                    self.global_colors.push(color);
                };
            });
            if ui.checkbox(&mut self.mark_every_color,"Select every color").clicked(){
                for c in &mut self.global_colors.iter_mut(){
                   c.marked = self.mark_every_color; 
                }
            };
            egui::ScrollArea::vertical().max_height(ui.available_height()-24.0).auto_shrink([false,true]).show(ui, |ui| {
                let aw = ui.available_width();
                egui::Grid::new("global_Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                    let mut column_count = 0;
                    for c in self.global_colors.iter_mut(){
                        iris_color::color_display(ui,c);
                        column_count += 1;
                        if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                            ui.end_row();
                            column_count = 0;
                        }
                    }
                });
            });
            ui.spacing();
            if ui.button("Compare").on_hover_text("Compare selected colors").clicked(){
                self.compare_window.push(ColorCompareWindow::new(self.get_selected_colors()));
            }
            let mut compare_window_to_delete:Vec<usize> = vec![];
            for (index,w) in self.compare_window.iter_mut().enumerate() {
                if w.window_open {
                    w.show(ctx);
                }else{
                    compare_window_to_delete.push(index);
                }
            }
            for i in compare_window_to_delete {
                self.compare_window.remove(i);
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut image_window_to_remove:Vec<usize> = vec![];
            let mut image_creation_windows_to_remove:Vec<usize> = vec![];
            for (index,w) in self.image_windows.iter_mut().enumerate(){
                if w.open {
                    w.show(ui.ctx());
                }else{
                   image_window_to_remove.push(index); 
                }
            }
            for (index,w) in self.image_creation_windows.iter_mut().enumerate(){
                if w.open {
                    w.show(ui.ctx());
                }else{
                   image_creation_windows_to_remove.push(index); 
                }
            }
            for index in image_window_to_remove{
                self.image_windows.remove(index);
            }
        }); 
        for color in self.global_colors.iter_mut() {
            color.color_info_window_show(ctx);
        }
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

struct ColorCompareWindow {
    img:iris_image_creation::PieColorComp,
    texture:Option<egui::TextureHandle>,
    colors:Vec<iris_color::AvarageRgb>,
    id:usize,
    window_open:bool,
}

impl ColorCompareWindow {
    fn new(colors:Vec<iris_color::AvarageRgb>) -> Self {
        WINDOW_ID.with(|thread_id|{
            let img = iris_image_creation::PieColorComp::new(colors.clone(),128);
            let texture:Option<egui::TextureHandle> = None;
            let id = thread_id.get();
            thread_id.set(id+1);
            Self{
                img,
                texture,
                colors,
                id,
                window_open:true,
            }
        })
    } 
    fn show(&mut self,ctx:&egui::Context){
        if self.texture.is_none(){
            self.img.generate_pie();
            self.texture = Some(ctx.load_texture("comp_img",ColorImage::from_rgb([self.img.size.try_into().unwrap(),self.img.size.try_into().unwrap()],&self.img.img),Default::default()));
        } 
        if self.window_open {
            let mut window_open_buffer = self.window_open;
            egui::Window::new("Compare").id(egui::Id::new(self.id)).open(&mut window_open_buffer).show(ctx,|ui|{
                if let Some(texture) = &self.texture {
                    ui.add(
                        egui::Image::from_texture(texture)
                    );
                }
            });
            self.window_open = window_open_buffer;
        } 
    }
}
    
