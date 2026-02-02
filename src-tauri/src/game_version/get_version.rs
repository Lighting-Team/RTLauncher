pub mod get_version {
    const version_suffix: &str = "/mc/game/version_manifest.json"; // http://launchermeta.mojang.com/mc/game/version_manifest.json
    const version2_suffix: &str = "/mc/game/version_manifest_v2.json"; // http://launchermeta.mojang.com/mc/game/version_manifest_v2.json

    pub mod heads {
        pub mod mojang_heads {
            const launcher: &str = "https://launcher.mojang.com";
            const launcher_meta: &str = "https://launchermeta.mojang.com";
            const assests: &str = "http://resources.download.minecraft.net";
            const libraries: &str = "https://libraries.minecraft.net";
        }

        pub mod mirror_heads {
            const bmclapi: &str = "https://bmclapi2.bangbang93.com"; // BMCLAPI 镜像站
            const jcut: &str = "https://mirrors.jcut.edu.cn/bmclapi"; // 荆楚理工学院镜像站
            const lzuoss: &str = "https://mirror.lzu.edu.cn/bmclapi"; // 兰州大学镜像站
            const nju: &str = "https://mirror.nju.edu.cn/bmclapi"; // 南京大学镜像站
            const nyist: &str = "https://mirror.nyist.edu.cn/bmclapi"; // 南阳理工学院镜像站
            const qlut: &str = "https://mirrors.qlu.edu.cn/bmclapi"; // 齐鲁工业大学镜像站
            const sjtug: &str = "https://mirror.sjtu.edu.cn/bmclapi"; // 思源镜像站
            const ustc: &str = "https://mirrors.ustc.edu.cn/bmclapi"; // 中国科学技术大学镜像站
        }

        pub mod authlib_injector {
            const official: &str = "https://authlib-injector.yushi.moe";
            const bmclapi: &str = "https://bmclapi2.bangbang93.com/mirrors/authlib-injector";
        }

        pub mod mod_loaders {
            pub mod forge {
                const official: &str = "https://files.minecraftforge.net/maven";
                const bmclapi: &str = "https://bmclapi2.bangbang93.com/maven";
            }

            pub mod fabric {
                const meta: &str = "https://meta.fabricmc.net";
                const bmcl_meta: &str = "https://bmclapi2.bangbang93.com/fabric-meta";
                const maven: &str = "https://maven.fabricmc.net";
                const bmcl_maven: &str = "https://bmclapi2.bangbang93.com/maven";
            }

            pub mod liteloader {
                const official: &str = "http://dl.liteloader.com/versions/versions.json";
                const bmclapi: &str =
                    "https://bmclapi.bangbang93.com/maven/com/mumfrey/liteloader/versions.json";
            }

            pub mod neoforge {
                const forge: &str = "https://maven.neoforged.net/releases/net/neoforged/forge";
                const bmcl_forge: &str =
                    "https://bmclapi2.bangbang93.com/maven/net/neoforged/forge";
                const neoforge: &str =
                    "https://maven.neoforged.net/releases/net/neoforged/neoforge";
                const bmcl_neoforge: &str =
                    "https://bmclapi2.bangbang93.com/maven/net/neoforged/neoforge";
            }

            pub mod quilt {
                const maven: &str = "https://maven.quiltmc.org/repository/release";
                const bmcl_maven: &str = "https://bmclapi2.bangbang93.com/maven";
                const meta: &str = "https://meta.quiltmc.org";
                const bmcl_meta: &str = "https://bmclapi2.bangbang93.com/quilt-meta";
            }

            mod cyan {
                // https://www.mcmod.cn/class/4420.html
            }

            mod flint {
                // https://www.mcmod.cn/class/14621.html
            }

            mod m3l {
                // https://www.mcmod.cn/class/19181.html
            }
        }
    }

    pub fn get_version_manifest_link(mirror: &str) -> &'static str {
        match mirror {
            "mojang" => heads::mojang_heads::launcher_meta + version_suffix,
            "mojang_v2" => heads::mojang_heads::launcher_meta + version2_suffix,
            "bmclapi" => heads::mirror_heads::bmclapi + version_suffix,
            "bmclapi_v2" => heads::mirror_heads::bmclapi + version2_suffix,
            "jcut" => heads::mirror_heads::jcut + version_suffix,
            "jcut_v2" => heads::mirror_heads::jcut + version2_suffix,
            "lzuoss" => heads::mirror_heads::lzuoss + version_suffix,
            "lzuoss_v2" => heads::mirror_heads::lzuoss + version2_suffix,
            "nju" => heads::mirror_heads::nju + version_suffix,
            "nju_v2" => heads::mirror_heads::nju + version2_suffix,
            "nyist" => heads::mirror_heads::nyist + version_suffix,
            "nyist_v2" => heads::mirror_heads::nyist + version2_suffix,
            "qlut" => heads::mirror_heads::qlut + version_suffix,
            "qlut_v2" => heads::mirror_heads::qlut + version2_suffix,
            "sjtug" => heads::mirror_heads::sjtug + version_suffix,
            "sjtug_v2" => heads::mirror_heads::sjtug + version2_suffix,
            "ustc" => heads::mirror_heads::ustc + version_suffix,
            "ustc_v2" => heads::mirror_heads::ustc + version2_suffix,
            _ => heads::mojang_heads::launcher_meta + version_suffix,
        }
    }

    pub fn get_assests(mirror: &str) -> &'static str {
        match mirror {
            "bmclapi" => heads::mirror_heads::bmclapi + "/assets",
            "jcut" => heads::mirror_heads::jcut + "/assets",
            "lzuoss" => heads::mirror_heads::lzuoss + "/assets",
            "nju" => heads::mirror_heads::nju + "/assets",
            "nyist" => heads::mirror_heads::nyist + "/assets",
            "qlut" => heads::mirror_heads::qlut + "/assets",
            "sjtug" => heads::mirror_heads::sjtug + "/assets",
            "ustc" => heads::mirror_heads::ustc + "/assets",
            _ => heads::mojang_heads::assests,
        }
    }

    #[derive(Debug)]
    pub enum VersionError {
        NetworkError(String),
        ParseError(String),
        InvalidMirror(String),
    }

    impl std::fmt::Display for VersionError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                VersionError::NetworkError(e) => write!(f, "网络请求失败: {}", e),
                VersionError::ParseError(e) => write!(f, "数据解析失败: {}", e),
                VersionError::InvalidMirror(e) => write!(f, "无效的镜像源: {}", e),
            }
        }
    }

    impl std::error::Error for VersionError {}

    pub struct VersionManifestParams {
        pub use_version2: bool,
        pub timeout_seconds: u64,
    }

    impl Default for VersionManifestParams {
        fn default() -> Self {
            Self {
                use_version2: true,
                timeout_seconds: 3,
            }
        }
    }

    /// 统一的版本清单获取函数
    pub async fn fetch_version_manifest(
        source: MirrorSource,
        params: Option<VersionManifestParams>,
    ) -> Result<String, VersionError> {
        let params = params.unwrap_or_default();

        let base_url = source.base_url();
        let url = if params.use_version2 {
            format!("{}{}", base_url, version2_suffix)
        } else {
            format!("{}{}", base_url, version_suffix)
        };

        match fetch_json(&url, Some(params.timeout_seconds)).await {
            Ok(content) => Ok(content),
            Err(e) => Err(VersionError::NetworkError(e.to_string())),
        }
    }

    pub enum MirrorSource {
        Mojang { use_v2: bool },
        Bmclapi { use_v2: bool },
        Jcut { use_v2: bool },
        Lzuoss { use_v2: bool },
        Nju { use_v2: bool },
        Nyist { use_v2: bool },
        Qlut { use_v2: bool },
        Sjtug { use_v2: bool },
        Ustc { use_v2: bool },
    }

    impl MirrorSource {
        fn base_url(&self) -> &str {
            match self {
                MirrorSource::Mojang { use_v2: _ } => &heads::mojang_heads::launcher_meta,
                MirrorSource::Bmclapi { use_v2: _ } => &heads::mirror_heads::bmclapi,
                MirrorSource::Jcut { use_v2: _ } => &heads::mirror_heads::jcut,
                MirrorSource::Lzuoss { use_v2: _ } => &heads::mirror_heads::lzuoss,
                MirrorSource::Nju { use_v2: _ } => &heads::mirror_heads::nju,
                MirrorSource::Nyist { use_v2: _ } => &heads::mirror_heads::nyist,
                MirrorSource::Qlut { use_v2: _ } => &heads::mirror_heads::qlut,
                MirrorSource::Sjtug { use_v2: _ } => &heads::mirror_heads::sjtug,
                MirrorSource::Ustc { use_v2: _ } => &heads::mirror_heads::ustc,
            }
        }
    }

    async fn fetch_json(url: &str, timeout_seconds: Option<u64>) -> Result<String, VersionError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds.unwrap_or(30)))
            .pool_max_idle_per_host(5)
            .gzip(true)
            .brotli(true)
            .build()
            .map_err(|e| VersionError::NetworkError(e.to_string()))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| VersionError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VersionError::NetworkError(format!(
                "HTTP错误: {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| VersionError::ParseError(e.to_string()))?;

        Ok(content)
    }
}
