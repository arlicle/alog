use alog::{render_site, watch_site_change, run_site_server, get_site_settings};


fn main() {

    let config = get_site_settings();

    // 建立异步线程，监控文件改动，当改动的时候，就重新生成站点
    watch_site_change(config.clone());

    render_site(config.clone());

    run_site_server(config.clone());
}


