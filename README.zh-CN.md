<div align="center">

# Brain Fucker

语言：[English](README.md)，[简体中文](README.zh-CN.md)。

</div>

## 环境要求

构建过程需要 [Rust 工具链](https://rust-lang.org/tools/install/)。

## 编译和运行

在项目根目录下执行
```bash
cargo build
```
可以加上 `--release` 参数以使用发布模式构建项目。

用法
```bash
$ ./bf-interpreter --help
Usage: bf-interpreter <FILENAME>

Arguments:
  <FILENAME>  

Options:
  -h, --help     Print help
  -V, --version  Print version
```
例如，可以执行
```bash
./bf-interpreter code.bf
```
来解释运行 `code.bf` 代码。解释器分别**以自身的标准输入、标准输出作为 BrainFuck 代码的标准输入和标准输出**。

可以执行（加上 `--release` 性能更优）
```bash
cargo test
```
测试赛题给出的 7 组样例。

## 功能

1. 解析命令行参数。可以使用 `bf-interpreter --help` 查看使用方式。
2. 从文件中读取 BrainFuck 代码（忽略非 BrainFuck 有效字符），以 `Instr` 枚举类型保存。
3. 指令折叠优化：折叠相邻的同类型指令。
4. 模式识别（idiom recognize）优化：识别常用的循环模式，替换为扩展指令。
5. 预处理跳转位置：对每个 `[` 和 `]` 指令预处理出跳转的位置。
6. 解释执行。

## AI 使用说明

- 使用 Copilot 进行短片段补全。**所有**代码均以手敲 + Copilot 补全的形式完成。
- 使用网页端 Gemini 3.1 Pro 查询优化思路、寻找 bug 和翻译本文档。