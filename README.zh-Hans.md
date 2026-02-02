<div align="center">

# TranslateGemma Desktop

TranslateGemma 的跨平台桌面应用，基于 [GPUI](https://www.gpui.rs/) 构建。

<samp>

**[English](README.md)** ┃ **[简体中文](README.zh-Hans.md)**

</samp>

</div>

> [!WARNING]
> **开发中 (WIP)**
>
> 本项目尚处于早期开发阶段，当前版本仅供预览与测试。

> [!NOTE]
> TranslateGemma 是基于 Gemma 3 构建的全新开放翻译模型系列，支持 55 种语言的跨语言沟通，旨在让人们无论身在何处、使用何种设备，都能无障碍交流。

## 安装

### 下载

请前往 [Releases](https://github.com/fhluo/translate-gemma-desktop/releases) 页面下载最新版本。

### 模型设置

使用前请先安装 [Ollama](https://ollama.com/)，并运行以下命令拉取 TranslateGemma 模型：

```shell
ollama pull translategemma
```

您也可以根据需要选择其他模型变体：

- `translategemma:12b`
- `translategemma:27b`
