# alog
一款用Rust写的生成静态网站的程序。


## 安装说明
### 直接下载程序运行
1 Mac os 系统

2 Linux 系统

3 Windows系统

1 下载源码后，编译后，访问`http:localhost:7878 `, 7878为默认端口，可以在程序自动生成的配置文件`config.toml`中修改。

### 写markdown日志
写日志目前只支持markdown语法，文件放在md目录中，也可以在``config.toml`文件中修改。因为以后文件会越来越多，为了方便维护和管理日志，建议文件按照文件夹`年/月/`的方式建立来写，或者自己定义的其它方式都可以，程序会自动找出所有md文件，然后根据时间去生成。

写日志的时候，如果文件名按照这么来写`20191001-Hello World.md`, 内容为
```html
My content is Hello World!
```

那么程序渲染时会把20191001作为日志发布时间：2019-10-01来用，Hello World作为日志的标题。时间这里也可以写到具体几点几分写,例如`201910011430-Hello World.md`

也可以在日志文件中来指定写的时间，以及标题。例如日志：`First post`，内容为：
```html
<!--
      {
    "title": "My First blog post",
    "post_date": "2019-09-10 12:39",
    "url": "my-first-post"
}
-->
This is my first post.

Wohoo!
```
其中`<!--{`开始到 `}-->`这个部分的全部内容，为这篇post的配置文件，可以配置他的标题 title, 发布时间 post_date, 网址 my-first-post

### 发布到github、Coding、码云

### 自定义皮肤

皮肤都放在theme文件夹中，可以复制default皮肤出来改。启用皮肤前，需要到`config.toml`中修改配置。

界面展示

![avatar](/theme/default/img/a1.png)
![avatar](/theme/default/img/ax.png)
