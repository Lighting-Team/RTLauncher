pub mod get_origin_version {
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

            mod cyan{
                // https://www.mcmod.cn/class/4420.html
            }

            mod flint{
                // https://www.mcmod.cn/class/14621.html
            }

            mod m3l{
                // https://www.mcmod.cn/class/19181.html
            }
        }
    }
}
