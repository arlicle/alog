use alog::{render_site, watch_site_change, run_site_server, get_site_settings};


fn main() {

    let a:Vec<i32> = (1..100).collect();
    let mut current_start = 0;
    let mut current_end = 0;
    let mut current_page = 0;
    let length = a.len();
    let per_page = 15;
    loop {
        current_start = current_page * per_page;
        let current_end = current_start+per_page;
        if current_end < length {
            let data = &a[current_start..current_end];
            println!("{:?}", data);
            current_page += 1;
        } else {
            let data = &a[current_start..length];
            println!("{:?}", data);
            break
        }
    }
    let b = &a[0..14];

    println!("{:?}", b);
    let config = get_site_settings();

    // 建立异步线程，监控文件改动，当改动的时候，就重新生成站点
    watch_site_change(config.clone());

    render_site(config.clone());

    run_site_server(config.clone());
}


