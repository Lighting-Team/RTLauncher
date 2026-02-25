//! 下载源模块
//!
//! 提供下载源管理和选择功能

/// 获取启动器或元数据下载源
/// 根据原始URL生成官方源和镜像源的URL列表
pub fn dl_source_launcher_or_meta_get(original_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    
    // 添加原始URL（通常是官方源）
    urls.push(original_url.to_string());
    
    // 如果原始URL不是镜像源，添加镜像源
    if !original_url.contains("bmclapi") && !original_url.contains("mcbbs") {
        // 转换官方URL为镜像URL
        let mirror_url = if original_url.contains("launchermeta.mojang.com") {
            original_url.replace("https://launchermeta.mojang.com", "https://bmclapi2.bangbang93.com")
        } else if original_url.contains("launcher.mojang.com") {
            original_url.replace("https://launcher.mojang.com", "https://bmclapi2.bangbang93.com")
        } else if original_url.contains("resources.download.minecraft.net") {
            original_url.replace("https://resources.download.minecraft.net", "https://bmclapi2.bangbang93.com/assets")
        } else if original_url.contains("libraries.minecraft.net") {
            original_url.replace("https://libraries.minecraft.net", "https://bmclapi2.bangbang93.com/maven")
        } else {
            original_url.to_string()
        };
        
        if mirror_url != original_url {
            urls.push(mirror_url);
        }
    }
    
    urls
}

/// 获取官方源
pub fn dl_source_official() -> String {
    "https://launchermeta.mojang.com".to_string()
}

/// 获取镜像源
pub fn dl_source_mirror() -> String {
    "https://bmclapi2.bangbang93.com".to_string()
}
