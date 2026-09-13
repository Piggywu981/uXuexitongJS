# 架构概览

uXueScript 由 Vue 前端、Tauri/Rust 后端和课程 WebView 组成。课程自动化逻辑保留为独立的 `core.js` 文件，可由桌面端注入，也可直接复制到浏览器控制台执行。

```mermaid
flowchart TB
    %% 外部生态
    subgraph External["外部服务与模型生态 (External Services)"]
        direction LR
        CXServer["超星学习通服务器<br/>(mooc1 / mooc2 / passport2)"]
        LLMCloud["各大 LLM 供应商 API<br/>(DeepSeek / OpenAI / Gemini / 智谱 等)"]
        OllamaLocal["本地 Ollama 实例<br/>(http://localhost:11434)"]
    end

    %% 前端层
    subgraph Frontend["前端展示与交互层 (Vue 3 + TS)"]
        direction TB
        subgraph Windows["多 Webview 交互视图"]
            MainView["主控制面板 (TheMainPage.vue)<br/>- 课程进度看板 (TheCourseDashboard)<br/>- 页面控制器 (TheChaoxingWebviewController)<br/>- 模型与参数配置 (TheConfigLayout)"]
            MaskView["动画遮罩层 (TheMaskPage.vue)<br/>- 启动开场动画 & 退出拦截淡出"]
        end
        SpectaClient["类型安全 IPC 客户端 (cmds.ts)"]
    end

    %% 后端核心层
    subgraph Backend["Rust / Tauri 2 宿主与调度核心 (Host Runtime)"]
        direction TB
        MacroHandler["编译期命令分发器<br/>(commands_collector 宏)"]

        subgraph HostSubsystems["核心系统调度模块"]
            direction LR
            WindowEngine["窗口几何管理 (app::window / webview)<br/>- 齐次比例自适应布局 (0.51W / 0.46H)<br/>- 导航历史栈 (UrlStack)"]
            RouteEngine["页面分类与脚本调度 (core::url / script)<br/>- URL 模式特征分类<br/>- 动态脚本按需注入 (eval)"]
            ConfigStore["纯 DTO 配置管理 (config)<br/>- API Key 脱敏与持久化"]
        end

        subgraph QuizEngine["测验逆向与求解管道 (core::quiz)"]
            direction TB
            FontParser["字体逆向引擎 (typr.rs & mapper.rs)<br/>1. 提取 @font-face font-cxsecret Base64 TTF<br/>2. 解析 loca & glyf 表提取贝塞尔矢量轮廓<br/>3. MD5 特征哈希检索 table.json 还原明文"]
            LLMDispatcher["LLM 并发分发器 (dispatcher.rs)<br/>- 题目切片 (CHUNK_SIZE = 5)<br/>- 信号量限流 (MAX_CONCURRENCY = 10)<br/>- 429 速率限制重试 (Retry-After / 指数退避)"]
        end
    end

    %% 自动化运行时层
    subgraph WebviewContext["内嵌课程运行环境 (Webview: 'chaoxing')"]
        direction TB
        CoreEngine["core.js 自动化引擎 (自包含单文件契约)"]

        subgraph CoreSub["自动化流水线"]
            direction LR
            StateGuard["后台运行与失焦守护<br/>- 拦截 blur / visibilitychange 暂停<br/>- 保持 document.hidden=false 活跃态"]
            DOMWalker["DOM 穿透探测器<br/>- 章节树遍历 & Tab 页签轮询<br/>- 多层嵌套 iframe 递归穿透"]
            TaskPipeline["任务执行步进器<br/>- Video: 速率锁定 & 静音<br/>- PDF: 平滑滚动触底<br/>- Quiz: UEditor 富文本注入<br/>- MutationObserver 监听完成"]
        end
    end

    %% 数据与通信流向
    MainView --> SpectaClient
    SpectaClient <-->|Tauri IPC (invoke)| MacroHandler
    MaskView <-->|窗口事件 (emit / listen)| MacroHandler

    MacroHandler --> HostSubsystems
    MacroHandler --> QuizEngine

    WindowEngine -.->|齐次比例几何贴合 / 缩放| WebviewContext
    RouteEngine ==>|页面加载完毕后动态注入| CoreEngine

    CoreEngine --> StateGuard
    CoreEngine --> DOMWalker
    DOMWalker --> TaskPipeline

    TaskPipeline -->|1. 提取加密 HTML 题目 (solve_quiz)| MacroHandler
    MacroHandler -->|调用解密| FontParser
    FontParser -->|还原明文题目列表| LLMDispatcher
    LLMDispatcher <==>|HTTP POST 推理请求| LLMCloud
    LLMDispatcher <==>|本地 HTTP API| OllamaLocal
    LLMDispatcher -->|2. 返回结构化答案数组| TaskPipeline

    TaskPipeline -.->|3. 发送进度状态 (send_status)| MacroHandler
    MacroHandler -.->|status-update 事件广播| MainView

    WebviewContext <==>|加载课程与音视频资源| CXServer
```

## 前端

`src` 包含 Vue 页面、布局、组件和由 Tauri Specta 生成的命令绑定。Configuration 管理模型和课程选项；课程仪表盘订阅后端状态事件；顶部控制栏负责课程 WebView 的导航和缩放。

## 后端与 WebView

`src-tauri/src` 负责窗口、配置、命令和自动化逻辑。课程 WebView 访问学习通页面；后端在页面加载完成后按 URL 类型注入 `core.js`、课程信息读取脚本或登录辅助脚本。

## 独立脚本

`src-tauri/src/scripts/core.js` 是自包含交付文件，不依赖 ES module 导入，以保留直接复制到浏览器控制台执行的能力。无后端使用方式见[无后端模式](../backend-free.md)。

## 测验与模型

`src-tauri/src/core/quiz` 负责 HTML 提取、加密字体映射和模型请求。该模块依赖桌面端后端，不属于独立脚本模式。
