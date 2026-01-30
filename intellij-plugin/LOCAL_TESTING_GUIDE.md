# 本地测试和打包指南

## 前提条件

### 必需软件
- ✅ JDK 17 或更高版本
- ✅ Gradle 8.5+ (或使用项目自带的 Gradle Wrapper)
- ✅ Rust 工具链 (用于构建音频核心)

### 检查环境

```bash
# 检查 Java 版本
java -version
# 应该显示 17 或更高

# 检查 Gradle (可选，可以使用 wrapper)
gradle --version

# 检查 Rust
rustc --version
cargo --version
```

## 步骤 1: 构建 Rust 音频核心

插件依赖 Rust 音频引擎，需要先构建原生库。

```bash
# 进入 Rust 核心目录
cd core

# 构建 Release 版本
cargo build --release

# 构建完成后，库文件位于：
# - Linux: target/release/libmusic_player_core.so
# - macOS: target/release/libmusic_player_core.dylib
# - Windows: target/release/music_player_core.dll
```

## 步骤 2: 复制原生库到插件目录

根据你的操作系统，复制对应的库文件：

### Linux (x64)
```bash
cd ../intellij-plugin
mkdir -p libs/linux-x64
cp ../target/release/libcontextune_core.so libs/linux-x64/
```

### macOS (Intel)
```bash
cd ../intellij-plugin
mkdir -p libs/macos-x64
cp ../target/release/libcontextune_core.dylib libs/macos-x64/
```

### macOS (Apple Silicon)
```bash
cd ../intellij-plugin
mkdir -p libs/macos-aarch64
cp ../target/release/libcontextune_core.dylib libs/macos-aarch64/
```

### Windows (x64)
```powershell
cd ..\intellij-plugin
mkdir libs\windows-x64
copy ..\target\release\contextune_core.dll libs\windows-x64\
```

### 使用自动化脚本 (推荐)

项目提供了自动化脚本：

```bash
cd intellij-plugin
./build.sh
```

这个脚本会自动：
1. 检测操作系统
2. 复制对应的原生库
3. 构建插件

## 步骤 3: 构建插件

### 方法 1: 使用 Gradle Wrapper (推荐)

```bash
cd intellij-plugin

# 清理之前的构建
./gradlew clean

# 构建插件
./gradlew buildPlugin
```

### 方法 2: 使用系统 Gradle

```bash
cd intellij-plugin
gradle clean buildPlugin
```

### 构建输出

构建成功后，插件 ZIP 文件位于：
```
intellij-plugin/build/distributions/Contexture Music Player-0.1.0.zip
```

## 步骤 4: 在开发模式下测试

在打包安装之前，可以先在开发模式下测试：

```bash
cd intellij-plugin

# 启动带插件的 IntelliJ IDEA 实例
./gradlew runIde
```

这会：
1. 下载 IntelliJ IDEA Community Edition (如果还没有)
2. 安装你的插件
3. 启动一个新的 IDE 窗口

**测试步骤：**
1. IDE 启动后，查看右侧工具栏
2. 点击 "Contexture Music Player" 工具窗口
3. 测试播放控制、音量调节等功能
4. 使用键盘快捷键测试
5. 关闭 IDE，重新打开测试状态持久化

## 步骤 5: 安装到本地 IDE

### 方法 1: 从磁盘安装 (推荐)

1. 打开你的 IntelliJ IDEA
2. 进入 `Settings/Preferences` (macOS: `⌘,`, Windows/Linux: `Ctrl+Alt+S`)
3. 选择 `Plugins`
4. 点击齿轮图标 ⚙️
5. 选择 `Install Plugin from Disk...`
6. 浏览到 `intellij-plugin/build/distributions/Contexture Music Player-0.1.0.zip`
7. 点击 `OK`
8. 重启 IDE

### 方法 2: 使用 Gradle 任务安装

```bash
# 构建并准备安装
./gradlew buildPlugin

# 然后手动安装 ZIP 文件（见方法 1）
```

## 步骤 6: 验证安装

重启 IDE 后：

1. **检查插件是否已安装：**
   - `Settings` → `Plugins` → `Installed`
   - 查找 "Contexture Music Player"

2. **打开工具窗口：**
   - `View` → `Tool Windows` → `Contexture Music Player`
   - 或点击右侧工具栏的图标

3. **测试功能：**
   - 查看 UI 是否正常显示
   - 测试播放控制按钮
   - 调节音量滑块
   - 测试键盘快捷键

## 步骤 7: 测试功能

### 基础功能测试

```
✓ 工具窗口打开/关闭
✓ UI 组件显示正常
✓ 播放/暂停按钮
✓ 停止按钮
✓ 音量滑块 (0-100%)
✓ 静音按钮
✓ 进度条显示
```

### 键盘快捷键测试

```
✓ Ctrl+Alt+P - 播放/暂停
✓ Ctrl+Alt+S - 停止
✓ Ctrl+Alt+N - 下一曲
✓ Ctrl+Alt+B - 上一曲
✓ Ctrl+Alt+↑ - 音量增加
✓ Ctrl+Alt+↓ - 音量减少
✓ Ctrl+Alt+M - 静音/取消静音
```

### 状态持久化测试

1. 调整音量到 50%
2. 关闭 IDE
3. 重新打开 IDE
4. 打开工具窗口
5. ✓ 验证音量是否恢复到 50%

### 错误处理测试

1. 尝试加载不存在的文件
2. ✓ 应该显示错误通知
3. ✓ 插件继续正常工作

## 常见问题排查

### 问题 1: 找不到原生库

**错误信息：**
```
Failed to load native library
```

**解决方案：**
1. 确认已构建 Rust 核心：`cd core && cargo build --release`
2. 确认已复制库文件到 `libs/` 目录
3. 检查文件名和路径是否正确
4. 重新构建插件：`./gradlew clean buildPlugin`

### 问题 2: 插件无法加载

**错误信息：**
```
Plugin 'Contexture Music Player' failed to initialize
```

**解决方案：**
1. 检查 IDE 日志：`Help` → `Show Log in Finder/Explorer`
2. 查找错误堆栈
3. 确认 JDK 版本是 17+
4. 尝试在开发模式下运行：`./gradlew runIde`

### 问题 3: 构建失败

**错误信息：**
```
Execution failed for task ':buildPlugin'
```

**解决方案：**
1. 清理构建缓存：`./gradlew clean`
2. 删除 `.gradle` 和 `build` 目录
3. 重新构建：`./gradlew buildPlugin`
4. 检查 Gradle 版本：`./gradlew --version`

### 问题 4: IDE 版本不兼容

**错误信息：**
```
Plugin 'Contexture Music Player' is incompatible with this installation
```

**解决方案：**
1. 检查你的 IDE 版本
2. 插件支持：IntelliJ IDEA 2023.2 (build 232) 到 2024.2 (build 242.*)
3. 如需支持其他版本，修改 `build.gradle.kts` 中的 `sinceBuild` 和 `untilBuild`

## 高级测试

### 运行单元测试

```bash
cd intellij-plugin

# 运行所有测试
./gradlew test

# 查看测试报告
open build/reports/tests/test/index.html
```

### 运行插件验证器

验证插件与不同 IDE 版本的兼容性：

```bash
./gradlew runPluginVerifier
```

### 生成代码覆盖率报告

```bash
./gradlew test jacocoTestReport
open build/reports/jacoco/test/html/index.html
```

## 调试插件

### 在开发模式下调试

1. 在 IntelliJ IDEA 中打开插件项目
2. 创建 Gradle 运行配置：
   - Run → Edit Configurations
   - 添加 Gradle 配置
   - Tasks: `runIde`
3. 设置断点
4. 以调试模式运行

### 查看日志

**开发模式日志：**
```bash
# 日志输出在控制台
./gradlew runIde
```

**已安装插件日志：**
- macOS: `~/Library/Logs/JetBrains/IntelliJIdea2023.2/idea.log`
- Linux: `~/.cache/JetBrains/IntelliJIdea2023.2/log/idea.log`
- Windows: `%USERPROFILE%\AppData\Local\JetBrains\IntelliJIdea2023.2\log\idea.log`

或通过 IDE：`Help` → `Show Log in Finder/Explorer`

## 性能测试

### 内存使用

1. 打开 IDE
2. 启用插件
3. 使用一段时间
4. 检查内存使用：`Help` → `Diagnostic Tools` → `Memory Indicator`

### CPU 使用

1. 播放音乐
2. 观察 CPU 使用率
3. 应该保持在较低水平（< 5%）

## 卸载插件

### 从 IDE 卸载

1. `Settings` → `Plugins`
2. 找到 "Contexture Music Player"
3. 点击齿轮图标 → `Uninstall`
4. 重启 IDE

### 清理配置文件

```bash
# macOS/Linux
rm -rf ~/.config/JetBrains/IntelliJIdea2023.2/options/contextune-music-player.xml

# Windows
del %APPDATA%\JetBrains\IntelliJIdea2023.2\options\contextune-music-player.xml
```

## 发布准备

### 创建发布版本

1. 更新版本号：`gradle.properties` 中的 `pluginVersion`
2. 更新变更日志：`plugin.xml` 中的 `<change-notes>`
3. 构建：`./gradlew buildPlugin`
4. 测试 ZIP 文件
5. 创建 Git tag：`git tag v0.1.0`

### 发布到 JetBrains Marketplace (可选)

```bash
# 设置发布令牌
export PUBLISH_TOKEN=your_token_here

# 发布
./gradlew publishPlugin
```

## 快速参考

### 常用命令

```bash
# 清理
./gradlew clean

# 构建
./gradlew build

# 构建插件 ZIP
./gradlew buildPlugin

# 开发模式运行
./gradlew runIde

# 运行测试
./gradlew test

# 验证插件
./gradlew runPluginVerifier
```

### 文件位置

```
插件 ZIP:     build/distributions/Contexture Music Player-0.1.0.zip
测试报告:     build/reports/tests/test/index.html
原生库:       libs/{platform}/libmusic_player_core.{ext}
配置文件:     ~/.config/JetBrains/.../contextune-music-player.xml
日志文件:     ~/.cache/JetBrains/.../log/idea.log
```

## 总结

完整的测试流程：

```bash
# 1. 构建 Rust 核心
cd core && cargo build --release

# 2. 复制原生库并构建插件
cd ../intellij-plugin && ./build.sh

# 3. 开发模式测试
./gradlew runIde

# 4. 运行单元测试
./gradlew test

# 5. 构建最终 ZIP
./gradlew buildPlugin

# 6. 手动安装到 IDE 测试
# Settings → Plugins → Install from Disk → 选择 ZIP
```

现在你可以开始测试插件了！🎵
