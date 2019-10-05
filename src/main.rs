use alog::{get_md_files, get_posts, render_site, watch_site_change, run_site_server};


fn main() {

    let s = "aa\
    dddd \
    \
    \
    ";

    // 建立异步线程，监控文件改动，当改动的时候，就重新生成站点
    watch_site_change();

    render_site();

    run_site_server();
}


