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

            pub fn get_version(mirror: &str) -> &'static str {
                match mirror {
                    "mojang" => mojang_heads::launcher_meta + version_suffix,
                    "mojang_v2" => mojang_heads::launcher_meta + version2_suffix,
                    "bmclapi" => bmclapi + version_suffix,
                    "bmclapi_v2" => bmclapi + version2_suffix,
                    "jcut" => jcut + version_suffix,
                    "jcut_v2" => jcut + version2_suffix,
                    "lzuoss" => lzuoss + version_suffix,
                    "lzuoss_v2" => lzuoss + version2_suffix,
                    "nju" => nju + version_suffix,
                    "nju_v2" => nju + version2_suffix,
                    "nyist" => nyist + version_suffix,
                    "nyist_v2" => nyist + version2_suffix,
                    "qlut" => qlut + version_suffix,
                    "qlut_v2" => qlut + version2_suffix,
                    "sjtug" => sjtug + version_suffix,
                    "sjtug_v2" => sjtug + version2_suffix,
                    "ustc" => ustc + version_suffix,
                    "ustc_v2" => ustc + version2_suffix,
                    _ => mojang_heads::launcher_meta + version_suffix,
                }
            }

            pub fn get_assests(mirror: &str) -> &'static str {
                match mirror {
                    "bmclapi" => bmclapi + "/assets",
                    "jcut" => jcut + "/assets",
                    "lzuoss" => lzuoss + "/assets",
                    "nju" => nju + "/assets",
                    "nyist" => nyist + "/assets",
                    "qlut" => qlut + "/assets",
                    "sjtug" => sjtug + "/assets",
                    "ustc" => ustc + "/assets",
                    _ => mojang_heads::assests,
                }
            }
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

    async fn fetch_json(url: &str, timeout_seconds: Option<u64>) -> Result<String, Error> {
        // 创建可复用的 HTTP 客户端（启用连接池）
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds.unwrap_or(30)))
            .pool_max_idle_per_host(5)
            .gzip(true)
            .brotli(true)
            .build()?;

        // 发送请求并获取响应
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP错误: {}", response.status()),
            )));
        }

        let content = response.text().await?;

        Ok(content)
    }

    /// 镜像源枚举
    pub enum MirrorSource {
        Mojang,
        Bmclapi,
        Jcut,
        Lzuoss,
        Nju,
        Nyist,
        Qlut,
        Sjtug,
        Ustc,
    }

    impl MirrorSource {
        /// 获取镜像源的基础URL
        fn base_url(&self) -> &str {
            match self {
                MirrorSource::Mojang => &mojang_heads::launcher_meta,
                MirrorSource::Bmclapi => &mirror_heads::bmclapi,
                MirrorSource::Jcut => &mirror_heads::jcut,
                MirrorSource::Lzuoss => &mirror_heads::lzuoss,
                MirrorSource::Nju => &mirror_heads::nju,
                MirrorSource::Nyist => &mirror_heads::nyist,
                MirrorSource::Qlut => &mirror_heads::qlut,
                MirrorSource::Sjtug => &mirror_heads::sjtug,
                MirrorSource::Ustc => &mirror_heads::ustc,
            }
        }
    }

    /// 统一的版本清单获取函数
    pub fn fetch_version_manifest(
        source: MirrorSource,
        params: Option<(bool, Option<u64>)>,
    ) -> Result<String, Error> {
        let (use_version2, timeout_seconds) = params.unwrap_or((true, Some(3)));

        let base_url = source.base_url();
        let url = if use_version2 {
            base_url.to_string() + version2_suffix
        } else {
            base_url.to_string() + version_suffix
        };

        fetch_json(&url, timeout_seconds)
    }
}
