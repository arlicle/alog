#!/bin/bash

# 文章生成脚本 - 生成约100篇各种类型的中文文章

MD_DIR="md"
mkdir -p "$MD_DIR"

# 生成文章的函数
generate_article() {
    local date=$1
    local title=$2
    local category=$3
    local tags=$4
    local content=$5
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

$content
EOF
}

# 生成日期范围 (2024年1月到12月)
dates=(
    "2024-01-01" "2024-01-05" "2024-01-10" "2024-01-15" "2024-01-20" "2024-01-25" "2024-01-30"
    "2024-02-03" "2024-02-08" "2024-02-13" "2024-02-18" "2024-02-23" "2024-02-28"
    "2024-03-05" "2024-03-10" "2024-03-15" "2024-03-20" "2024-03-25" "2024-03-30"
    "2024-04-04" "2024-04-09" "2024-04-14" "2024-04-19" "2024-04-24" "2024-04-29"
    "2024-05-04" "2024-05-09" "2024-05-14" "2024-05-19" "2024-05-24" "2024-05-29"
    "2024-06-03" "2024-06-08" "2024-06-13" "2024-06-18" "2024-06-23" "2024-06-28"
    "2024-07-03" "2024-07-08" "2024-07-13" "2024-07-18" "2024-07-23" "2024-07-28"
    "2024-08-02" "2024-08-07" "2024-08-12" "2024-08-17" "2024-08-22" "2024-08-27"
    "2024-09-01" "2024-09-06" "2024-09-11" "2024-09-16" "2024-09-21" "2024-09-26"
    "2024-10-01" "2024-10-06" "2024-10-11" "2024-10-16" "2024-10-21" "2024-10-26" "2024-10-31"
    "2024-11-05" "2024-11-10" "2024-11-15" "2024-11-20" "2024-11-25" "2024-11-30"
    "2024-12-05" "2024-12-10" "2024-12-15" "2024-12-20" "2024-12-25" "2024-12-30"
)

counter=0

# 技术类文章
tech_articles=(
    "Rust异步编程最佳实践|编程|rust,async,编程"
    "微服务架构设计指南|技术|架构,微服务,设计"
    "深入理解TypeScript类型系统|编程|typescript,types,编程"
    "Docker容器化部署实战|技术|docker,部署,容器"
    "Python数据分析入门|编程|python,数据,分析"
    "Git工作流最佳实践|技术|git,workflow,版本控制"
    "React性能优化技巧|编程|react,性能,优化"
    "Vue3组合式API详解|编程|vue3,api,前端"
    "数据库索引优化策略|技术|数据库,索引,优化"
    "GraphQL vs RESTful API|技术|api,graphql,rest"
    "前端构建工具对比|编程|前端,构建,工具"
    "Rust所有权系统详解|编程|rust,所有权,内存"
    "WebAssembly未来发展|技术|wasm,web,未来"
    "Kubernetes集群管理|技术|kubernetes,集群,部署"
    "CI/CD流水线设计|技术|cicd,自动化,部署"
)

for article in "${tech_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 引言

本文将详细介绍${title##*}相关的核心概念和实践经验。

## 主要内容

### 概念理解

首先，我们需要理解相关的基本概念。${title##*}是一个非常重要的技术领域，它在现代软件开发中扮演着关键角色。

### 实际应用

在实际项目中，${title##*}可以帮助我们：

1. 提高开发效率
2. 优化系统性能
3. 增强代码可维护性

## 最佳实践

以下是一些最佳实践建议：

- 遵循项目规范
- 编写清晰的文档
- 进行充分的测试

## 总结

掌握${title##*}对于开发者来说是非常重要的，希望本文能够帮助你更好地理解和应用这项技术。
EOF
    ((counter++))
done

# 生活类文章
life_articles=(
    "2024年度旅行计划|生活|旅行,计划,生活"
    "健康饮食指南|生活|健康,饮食,营养"
    "读书笔记：如何高效阅读|生活|读书,阅读,笔记"
    "居家办公的效率提升|生活|远程工作,效率,办公"
    "摄影入门技巧分享|生活|摄影,技巧,艺术"
    "咖啡文化探索|生活|咖啡,文化,生活"
    "周末户外活动推荐|生活|户外,活动,周末"
    "极简主义生活方式|生活|极简,生活,理念"
    "植物种植入门|生活|园艺,种植,植物"
    "音乐欣赏指南|生活|音乐,艺术,欣赏"
)

for article in "${life_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 开场

${title##*}是我们日常生活中很重要的一部分，今天我想和大家分享一些心得。

## 为什么要关注${title##*}

在现代快节奏的生活中，关注${title##*}能够帮助我们：

- 放松身心
- 提升生活质量
- 获得更多乐趣

## 实践建议

以下是我的一些建议：

1. 从小事做起
2. 保持一致性
3. 享受过程

## 个人体验

分享一些我个人的体验和感受...

## 结语

希望这些建议对你有所帮助，让我们一起探索生活的美好。
EOF
    ((counter++))
done

# 读书笔记类
reading_articles=(
    "《代码整洁之道》读后感|读书|代码,整洁,质量"
    "《人月神话》项目管理启示|读书|管理,项目,神话"
    "《设计模式》学习笔记|读书|设计,模式,编程"
    "《算法导论》入门指南|读书|算法,编程,学习"
    "《黑客与画家》思想碰撞|读书|黑客,画家,思想"
    "《深入理解计算机系统》笔记|读书|计算机,系统,底层"
    "《重构》改善代码质量|读书|重构,代码,质量"
    "《敏捷软件开发》实践指南|读书|敏捷,开发,软件"
    "《人件》团队管理智慧|读书|团队,管理,人件"
    "《代码大全》编程艺术|读书|编程,代码,大全"
)

for article in "${reading_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 书籍简介

${title##*}是一本非常有价值的书籍，它涵盖了${title##*}的核心内容。

## 核心观点

书中提出了几个重要的观点：

- 观点一：详细阐述
- 观点二：深入分析
- 观点三：实践指导

## 个人感悟

阅读这本书让我对${title##*}有了更深刻的理解。

## 适用场景

这本书特别适合以下人群：

- 初学者
- 进阶开发者
- 团队管理者

## 推荐指数

⭐⭐⭐⭐⭐ 强烈推荐
EOF
    ((counter++))
done

# 工具类文章
tool_articles=(
    "VSCode高效使用技巧|工具|vscode,编辑器,效率"
    "Git高级用法详解|工具|git,版本控制,高级"
    "Docker常用命令总结|工具|docker,容器,命令"
    "Linux终端命令大全|工具|linux,终端,命令"
    "Markdown写作指南|工具|markdown,写作,文档"
    "PostgreSQL数据库管理|工具|postgresql,数据库,管理"
    "Redis缓存应用实践|工具|redis,缓存,应用"
    "Nginx配置优化指南|工具|nginx,配置,优化"
    "Webpack构建工具详解|工具|webpack,构建,前端"
    "Postman接口测试指南|工具|postman,测试,api"
)

for article in "${tool_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 简介

${title##*}是开发工作中必不可少的工具，掌握它的使用能大大提高工作效率。

## 基础用法

以下是${title##*}的基础用法：

\`\`\`bash
# 示例命令
command example
\`\`\`

## 高级技巧

一些高级使用技巧：

1. 技巧一
2. 技巧二
3. 技巧三

## 常见问题

Q: 常见问题一
A: 解决方案

## 资源推荐

- 官方文档
- 社区资源
- 在线教程
EOF
    ((counter++))
done

# 心得体会类
insight_articles=(
    "程序员职业发展规划|心得|职业,发展,规划"
    "技术债务管理经验|心得|技术,债务,管理"
    "代码审查的重要性|心得|代码,审查,质量"
    "持续学习的方法论|心得|学习,方法,持续"
    "团队协作的心得|心得|团队,协作,沟通"
    "远程工作的挑战与机遇|心得|远程,工作,挑战"
    "技术选型的思考|心得|技术,选型,决策"
    "项目复盘总结|心得|项目,复盘,总结"
    "压力管理与心理健康|心得|压力,健康,管理"
    "创新思维培养|心得|创新,思维,培养"
)

for article in "${insight_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 背景介绍

${title##*}是每个从业者都会遇到的问题，我想分享一些个人体会。

## 核心观点

我认为${title##*}的关键在于：

### 观点一

详细阐述...

### 观点二

详细阐述...

## 实践经验

以下是我的实践经验：

- 经验一
- 经验二
- 经验三

## 反思与改进

通过反思，我发现...

## 建议

给同行的建议：

1. 建议一
2. 建议二
3. 建议三
EOF
    ((counter++))
done

# 行业趋势类
trend_articles=(
    "2024年前端技术趋势|趋势|前端,趋势,2024"
    "后端架构发展方向|趋势|后端,架构,发展"
    "人工智能应用前景|趋势|ai,人工智能,应用"
    "云原生技术演进|趋势|云原生,技术,演进"
    "低代码平台发展|趋势|低代码,平台,发展"
    "DevOps未来展望|趋势|devops,未来,展望"
    "区块链技术应用|趋势|区块链,技术,应用"
    "边缘计算新机遇|趋势|边缘计算,机遇,技术"
    "5G时代的应用场景|趋势|5g,应用,场景"
    "量子计算探索|趋势|量子,计算,探索"
)

for article in "${trend_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 行业背景

${title##*}是当前技术行业的热门话题，值得我们深入探讨。

## 技术发展

### 现状

当前的发展状况...

### 趋势

未来的发展趋势...

## 应用场景

${title##*}的主要应用场景包括：

1. 场景一
2. 场景二
3. 场景三

## 挑战与机遇

面临的挑战：

- 挑战一
- 挑战二

带来的机遇：

- 机遇一
- 机遇二

## 总结

${title##*}将继续影响行业的发展，让我们拭目以待。
EOF
    ((counter++))
done

# 教程类文章
tutorial_articles=(
    "Rust入门教程系列（一）|教程|rust,入门,教程"
    "React组件开发实战|教程|react,组件,开发"
    "Python数据分析教程|教程|python,数据,分析"
    "Docker容器化教程|教程|docker,容器,教程"
    "Vue3项目实战教程|教程|vue3,项目,实战"
    "TypeScript进阶教程|教程|typescript,进阶,教程"
    "Linux系统管理教程|教程|linux,系统,管理"
    "Git版本控制教程|教程|git,版本,控制"
    "SQL数据库查询教程|教程|sql,数据库,查询"
    "网络协议基础教程|教程|网络,协议,基础"
)

for article in "${tutorial_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 教程简介

本教程将带你系统地学习${title##*}，从基础到实践。

## 前置知识

开始之前，你需要了解：

- 基础概念一
- 基础概念二

## 课程大纲

### 第一章：基础入门

内容介绍...

### 第二章：进阶应用

内容介绍...

### 第三章：实战项目

内容介绍...

## 学习资源

- 官方文档
- 在线教程
- 社区支持

## 练习题

完成本章学习后，尝试以下练习：

1. 练习一
2. 练习二

## 下一步

学习完本教程后，你可以继续深入学习...
EOF
    ((counter++))
done

# 添加更多技术类文章以接近100篇
more_tech_articles=(
    "Rust并发编程指南|编程|rust,并发,编程"
    "JavaScript闭包详解|编程|javascript,闭包,基础"
    "CSS Grid布局完全指南|编程|css,grid,布局"
    "HTTP协议详解|技术|http,协议,网络"
    "TCP/IP协议原理|技术|tcpip,协议,网络"
    "数据库事务处理|技术|数据库,事务,处理"
    "缓存策略设计|技术|缓存,策略,设计"
    "消息队列应用|技术|消息队列,应用,架构"
    "分布式系统设计|技术|分布式,系统,设计"
    "服务器性能优化|技术|服务器,性能,优化"
    "前端状态管理方案|编程|前端,状态,管理"
    "API设计最佳实践|技术|api,设计,最佳实践"
    "代码重构技巧|编程|重构,代码,技巧"
    "测试驱动开发|技术|tdd,测试,开发"
    "持续集成实践|技术|ci,集成,实践"
    "监控与日志管理|技术|监控,日志,管理"
    "安全防护措施|技术|安全,防护,措施"
    "移动端适配方案|技术|移动端,适配,方案"
    "搜索功能实现|技术|搜索,功能,实现"
    "实时通信技术|技术|实时,通信,技术"
)

for article in "${more_tech_articles[@]}"; do
    IFS='|' read -r title category tags <<< "$article"
    date=${dates[$counter % ${#dates[@]}]}
    
    cat > "$MD_DIR/$date-${title// /-}.md" << EOF
---
title: $title
date: $date
category: $category
tags: [$tags]
---

# $title

## 概述

${title##*}是现代软件开发中的重要主题，掌握它对于开发者来说至关重要。

## 基础概念

### 概念一

详细解释...

### 概念二

详细解释...

## 实现方式

### 方式一

\`\`\`
示例代码
\`\`\`

### 方式二

详细说明...

## 最佳实践

- 实践一
- 实践二
- 实践三

## 常见问题

Q: 常见问题
A: 解决方案

## 总结

${title##*}是一个值得深入研究的领域，希望本文对你有所帮助。
EOF
    ((counter++))
done

echo "已生成 $counter 篇文章"
echo "文章保存在 $MD_DIR 目录中"