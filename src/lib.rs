use walkdir::WalkDir;
use std::borrow::Borrow;
use std::io::Read;
use std::io::BufWriter;
use std::io::prelude::*;

use std::fs::File;
use regex::Regex;
use serde_json::{Result, Value as S_Value};
use serde_json::value::{to_value, Value};

use chrono::prelude::{DateTime, FixedOffset, TimeZone, Utc};
use serde_json::json;
use std::path::Path;
use pulldown_cmark::{Parser, html};
use std::collections::HashMap;

use tera::{Context, Result as Tera_Result, Tera, try_get_value};

use std::sync::{Mutex, Arc};
use std::thread;

use std::fs;

use std::net::TcpStream;
use std::net::TcpListener;
use urlencoding::decode;

use config::{Config, File as Config_File};


extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use serde::{Deserialize, Serialize, Serializer};

use crossbeam_channel::unbounded;
use notify::{RecommendedWatcher, RecursiveMode, Result as Notify_Result, Watcher};
use std::time::Duration;

pub fn watch_site_change(site_settings: Arc<Mutex<HashMap<String, String>>>) {
    // 建立异步线程，监控文件改动，当改动的时候，就重新生成站点
    thread::spawn(|| {
        let (tx, rx) = unbounded();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

        watcher.watch("md/", RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv() {
                Ok(event) => println!("changed: {:?}", event),
                Err(err) => println!("watch error: {:?}", err),
            };
        }
    });
}

pub fn get_site_settings() -> Arc<Mutex<HashMap<String, String>>> {
    let default_settings = "theme = \"default\"
source_posts_dir = \"md\"
site_title = \"Debug my self\"
static_html_dir = \"p\"
server_port = \"7878\"";

    let mut settings = Config::default();
    let config_file = Path::new("config.toml");

    // 判断是否有config.toml文件，如果没有就创建
    if !config_file.exists() {
        let f = File::create("config.toml").unwrap();
        {
            let mut writer = BufWriter::new(f);

            writer.write(default_settings.as_bytes()).unwrap();
        }
    }
    settings.merge(Config_File::from(config_file)).unwrap();
    let settings = settings.try_into::<HashMap<String, String>>().unwrap();
    Arc::new(Mutex::new(settings))
}

pub fn run_site_server(site_settings: Arc<Mutex<HashMap<String, String>>>) {
    let server_port = get_site_settings_val(site_settings, "server_port", "7878");
    let add = format!("127.0.0.1:{}", server_port);
    let listener = TcpListener::bind(&add).unwrap();
    println!("server is running on {}", &add);
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_server_connection(stream);
    }
}


fn handle_server_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let r = String::from_utf8_lossy(&buffer[..]).to_string();

    let mut contents = String::new();

    let re = Regex::new(r"^GET (.*?) ").unwrap();
    match re.captures(&r) {
        Some(cap) => {
            let url = match cap.get(1) {
                Some(u) => u.as_str().trim_start_matches("/"),
                None => panic!("can not get url"),
            };

            let filename = format!("{}index.html", url);
            let filename = match decode(&filename) {
                Ok(s) => s,
                Err(e) => filename
            };

            contents = match fs::read_to_string(filename) {
                Ok(s) => s,
                Err(e) =>
                    match fs::read_to_string(url) {
                        Ok(s) => s,
                        Err(e) => "<h1>404 File not found</h1>".to_string(),
                    }
            };
        }
        None => ()
    }

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn get_md_files(path: &str) -> Vec<String> {
    let mut result = Vec::new();
    for entry in WalkDir::new(path) {
        let e = entry.unwrap();
        let x = e.path().to_str().unwrap().to_owned();
        if x.ends_with(".md") {
            result.push(x);
        }
    }
    result
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostData {
    /// 某一个post的默认设置
    ///
    pub title: String,
    pub url: String,
    pub description: String,
    pub content: String,
    pub keywords: String,
    pub post_date: String,
    pub post_date_int: i64,
    pub author: String,
    pub prev_post: Box<Option<PostData>>,
    pub next_post: Box<Option<PostData>>,
}


impl PostData {
    pub fn new() -> Self {
        PostData {
            title: "".to_string(),
            url: "".to_string(),
            description: "".to_string(),
            content: "".to_string(),
            keywords: "".to_string(),
            post_date: "".to_string(),
            post_date_int: 0,
            author: "".to_string(),
            prev_post: Box::new(None),
            next_post: Box::new(None),
        }
    }
}


pub fn get_val_from_json(v: &Value, key: &str, default_val: String) -> String {
    /// 从json中获取对应字段的值
    /// 如果取不到值，那么保存为空
    match v.get(key) {
        Some(t) => {
            t.as_str().unwrap().to_owned()
        }
        None => default_val
    }
}


pub fn get_post_filename_as_url(file_path: &Path) -> String {
    let s = file_path.file_name().unwrap().to_str().unwrap();
    let s = s.trim_end_matches(".md");
    let re = Regex::new(r"^[\d\-]+-").unwrap();
    let after = re.replace_all(s, "");
    let after = after.replace("/", "_");
//    file_path.file_name().unwrap().to_str().unwrap().to_owned()
    after.to_string()
}

pub fn get_post_url(v: &Value, file_path: &Path) -> String {
    match v.get("url") {
        Some(t) => {
            t.as_str().unwrap().to_owned()
        }
        None => {
            get_post_filename_as_url(file_path)
        }
    }
}

pub fn parse_md_file(md_file: &str) -> PostData {
    /// 读取md文档里面的内容，变成一个post数据

    let file_path = Path::new(md_file);
    let mut f = File::open(file_path).unwrap();
    // 读取md文档内容
    let mut md_content = String::new();
    f.read_to_string(&mut md_content);
    let md_content_cleaned = md_content.trim();

    let mut post_data = PostData::new();

    post_data.content = md_content_cleaned.to_string();

    let file_created_time = f.metadata().unwrap().created().unwrap().elapsed().unwrap().as_secs();

    post_data.post_date_int = file_created_time as i64;
    let dt = Utc.timestamp(post_data.post_date_int, 0);
    let url = get_post_filename_as_url(file_path);
    post_data.url = format!("{}{}/", dt.format("p/%Y/%m/%d/").to_string(), url);
    post_data.title = url;


    // 获取md文档顶部的配置信息
    let re = Regex::new(r"\s*(<!--)?\s*(\{[\s\S]*?\})\s*(-->)?\s*").unwrap();
    let x = re.captures(&md_content);


    match re.captures(&md_content) {
        Some(captures) => {
            match captures.get(2) {
                Some(mat) => {
                    let post_config_str = mat.as_str();
                    let post_config_data: Value = serde_json::from_str(post_config_str).unwrap();

                    post_data.title = get_val_from_json(&post_config_data, "title", "".to_string());
                    post_data.post_date = get_val_from_json(&post_config_data, "post_date", "".to_string());
                    let dt = DateTime::parse_from_str(format!("{} +00:00", post_data.post_date).as_str(), "%Y-%m-%d %H:%M %z").expect("jjjjjj999999");
                    let t = dt.timestamp();
                    post_data.post_date_int = t;
                    post_data.url = format!("{}{}/", dt.format("p/%Y/%m/%d/").to_string(), get_post_url(&post_config_data, file_path));
                    let config_length = captures.get(0).unwrap().as_str().len();
                    post_data.content = (&md_content_cleaned[config_length..]).to_string();
                }
                None => ()
            }
        }
        None => ()
    };

    let parser = Parser::new(&post_data.content);
    let mut post_content_html = String::new();
    html::push_html(&mut post_content_html, parser);
    post_data.content = post_content_html;

    post_data
}


pub fn get_posts(path: &str) -> Vec<PostData> {
    /// 获取post 列表
    let mut post_list = Vec::new();
    let mds = get_md_files(path);
    for md in mds.iter() {
        let post = parse_md_file(md);
        post_list.push(post);
    }

    // 对post 按时间进行排序
    post_list.sort_unstable_by(|a, b| a.post_date_int.partial_cmp(&b.post_date_int).unwrap());

    // 对每一篇post生成 prev_post和next_post
    let l = post_list.len();
    let mut prev_post: Option<PostData> = None;
    let mut next_post: Option<PostData> = None;
    let mut new_post_list: Vec<PostData> = vec![];

    for (index, post) in post_list.iter().enumerate() {
        let mut post = post.clone();
        if index > 0 {
            post.prev_post = Box::new(prev_post);
        }
        if index < (l - 1) {
            post.next_post = Box::new(Some(post_list[index + 1].clone()));
        }
        prev_post = Some(post.clone());

        new_post_list.push(post);
    }

    new_post_list
}


pub fn render_site(site_settings: Arc<Mutex<HashMap<String, String>>>) {
    /// ## 生成静态网站，一共有4步
    /// 1. 删除旧的生成文件
    /// 2. 删除旧的静态文件
    /// 3. 生成新的静态文件
    /// 4. 拷贝新的静态文件

    // 1. 删除旧的生成文件

    // 2. 删除旧的静态文件

    // 3. 重新生成新的静态文件

    let mut tera = match Tera::new("theme/default/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("error is {:?}", e);
            ::std::process::exit(1);
        }
    };
//    tera.register_filter("do_nothing", do_nothing_filter);

    let site_title = get_site_settings_val(site_settings, "site_title", "alog");
    let mut context = Context::new();
    context.insert("site_title", &site_title);

    let tera = Arc::new(Mutex::new(tera));

    let posts = get_posts("md/");
    context.insert("post_list", &posts);
    let post = &posts[0];
    context.insert("first_post", post);

    let posts = Arc::new(Mutex::new(posts));

    let context = Arc::new(Mutex::new(context));

    let posts1 = Arc::clone(&posts);
    let context1 = Arc::clone(&context);
    let tera1 = Arc::clone(&tera);
    thread::spawn(move || {
        render_index_to_html(posts1, context1, tera1);
    });

    let posts1 = Arc::clone(&posts);
    let context1 = Arc::clone(&context);
    let tera1 = Arc::clone(&tera);
    thread::spawn(move || {
        render_per_post_to_html(posts1, context1, tera1);
    });


    let posts2 = Arc::clone(&posts);
    let context2 = Arc::clone(&context);
    let tera2 = Arc::clone(&tera);

    thread::spawn(move || {
        render_post_list_to_html(posts2, context2, tera2);
    });

    // 4. 拷贝新的静态文件
}


pub fn render_index_to_html(posts: Arc<Mutex<Vec<PostData>>>, context: Arc<Mutex<Context>>, tera: Arc<Mutex<Tera>>) {
    // 一个post一个post的渲染生成html

    let posts = posts.lock().unwrap();
    let mut context = context.lock().unwrap();
    let mut tera = tera.lock().unwrap();

    let x = tera.render("index.html", context.clone()).unwrap();

    let f = File::create("index.html").unwrap();
    {
        let mut writer = BufWriter::new(f);

        // write a byte to the buffer
        writer.write(x.as_bytes()).unwrap();
    } // the buffer is flushed once writer goes out of scope
}


pub fn render_per_post_to_html(posts: Arc<Mutex<Vec<PostData>>>, context: Arc<Mutex<Context>>, tera: Arc<Mutex<Tera>>) {
    // 一个post一个post的渲染生成html

    let posts = posts.lock().unwrap();
    let mut context = context.lock().unwrap();
    let mut tera = tera.lock().unwrap();

    for (index, post) in posts.iter().enumerate() {
        context.insert("post", &post);
        let x = tera.render("post.html", context.clone()).unwrap();

        match std::fs::create_dir_all(&post.url) {
            Ok(i) => (),
            Err(e) => ()
        }

        let f = File::create(format!("{}index.html", post.url)).unwrap();
        {
            let mut writer = BufWriter::new(f);

            // write a byte to the buffer
            writer.write(x.as_bytes()).unwrap();
        } // the buffer is flushed once writer goes out of scope
    }
}


pub fn render_post_list_to_html(posts: Arc<Mutex<Vec<PostData>>>, context: Arc<Mutex<Context>>, tera: Arc<Mutex<Tera>>) {
    // 生成html列表
    let posts = posts.lock().unwrap();
    let mut context = context.lock().unwrap();
    let mut tera = tera.lock().unwrap();

    let length = posts.len();


    let mut current_start = 0;
    let mut current_end = 0;
    let mut current_page = 1;
    let per_page = 1;
    let mut is_end = false;
    let mut page_post_list;

    let mut current_url = "".to_string();
    let mut next_url = "".to_string();
    let mut prev_url = "".to_string();

    // 计算页数
    let mut last_page_num = length / per_page;
    if length % per_page > 0 {
        last_page_num += 1;
    }
    let page_numbers: Vec<i32> = (1..=last_page_num as i32).collect();
    context.insert("last_page_num", &last_page_num);
    context.insert("page_numbers", &page_numbers);

    loop {
        current_start = (current_page - 1) * per_page;
        let current_end = current_start + per_page;
        current_url = format!("p/list/{}/", current_page);
        if current_page > 1 {
            prev_url = format!("p/list/{}/", current_page - 1);
        }
        if current_end < length {
            page_post_list = &posts[current_start..current_end];
            next_url = format!("p/list/{}/", current_page + 1);
        } else {
            page_post_list = &posts[current_start..length];
            is_end = true;
        }

        context.insert("current_page_post_list", &page_post_list);
        context.insert("current_page_num", &current_page);


        context.insert("current_page_url", &current_url);
        context.insert("next_page_url", &next_url);
        context.insert("prev_page_url", &prev_url);

        let x = tera.render("list.html", context.clone()).unwrap();


        match std::fs::create_dir_all(&current_url) {
            Ok(i) => (),
            Err(e) => ()
        }

        let f = File::create(format!("{}index.html", current_url)).unwrap();
        {
            let mut writer = BufWriter::new(f);
            writer.write(x.as_bytes()).unwrap();
        }

        if current_page == 1 {
            let f = File::create("p/list/index.html").unwrap();
            {
                let mut writer = BufWriter::new(f);
                writer.write(x.as_bytes()).unwrap();
            }
        }

        if (is_end) {
            break;
        }
        current_page += 1;
    }
}


fn get_site_settings_val(site_settings: Arc<Mutex<HashMap<String, String>>>, key: &str, default_val: &str) -> String {
    let s = site_settings.lock().unwrap();
    match s.get(key) {
        Some(v) => v.to_string(),
        None => default_val.to_string(),
    }
}

//pub fn do_nothing_filter(value: &Value, _: &HashMap<String, Value>) -> Tera_Result<Value> {
//    let s = try_get_value!("do_nothing_filter", "value", String, value);
//    Ok(to_value(&s).unwrap())
//}
