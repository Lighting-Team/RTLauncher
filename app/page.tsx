"use client"

import * as React from "react"
import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"
import { invoke } from "@tauri-apps/api/core"
import { toast } from "sonner"
import { AppSidebar } from "@/components/app-sidebar"
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbPage,
    BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { Separator } from "@/components/ui/separator"
import {
    SidebarInset,
    SidebarProvider,
    SidebarTrigger,
} from "@/components/ui/sidebar"

type ConfigStatus = "ok" | "missing" | "invalid_json" | "invalid_data" | "read_error" | "parse_error_initialized"
type AppConfig = {
    launcher: {
    initialized: boolean
    version: string
    generated_at: number
    }
}
type ConfigCheckResult = {
    status: ConfigStatus
    config?: AppConfig | null
    error?: string | null
}

async function verifyConfig() {
    try {
        const result = await invoke<ConfigCheckResult>("check_config_status")
        const isInitialized = result.config?.launcher?.initialized === true || result.status === "parse_error_initialized"
        
        return {
            isInitialized,
            status: result.status,
            error: result.error
        }
    } catch (error) {
        console.error("Config check failed", error)
        return {
            isInitialized: false,
            status: "read_error" as ConfigStatus,
            error: String(error)
        }
    }
}

export default function Page() {
  const router = useRouter()
  const [isReady, setIsReady] = useState(false)
  const [isChecking, setIsChecking] = useState(true)
  const [showRepairDialog, setShowRepairDialog] = useState(false)

  const processConfigResult = React.useCallback((result: { isInitialized: boolean, status: ConfigStatus, error?: string | null }) => {
        if (result.isInitialized) {
          if (result.status === "parse_error_initialized") {
             setShowRepairDialog(true)
             setIsReady(false)
          } else {
             setIsReady(true)
             if (result.status !== "ok") {
                 toast.warning("配置文件异常", {
                     description: result.error || "检测到配置文件可能存在数据缺失，请检查。",
                     duration: 5000,
                 })
             }
          }
        } else {
          router.push("/initialize")
        }
        setIsChecking(false)
  }, [router])

  useEffect(() => {
    let mounted = true
    
    const init = async () => {
        const result = await verifyConfig()
        if (mounted) {
            processConfigResult(result)
        }
    }
    
    init()
    
    return () => { mounted = false }
  }, [processConfigResult])

  const handleRepair = async () => {
       try {
           await invoke("repair_config")
           toast.success("配置文件修复成功")
           setShowRepairDialog(false)
           setIsChecking(true)
           
           // Re-check
           const result = await verifyConfig()
           processConfigResult(result)
       } catch (e) {
           console.error(e)
           toast.error("修复失败，请手动检查配置文件")
       }
   }

  if (isChecking) {
    return null
  }
  
  // 如果需要显示修复对话框，也渲染一个空的背景或者部分 UI，这里直接返回 null 等待 dialog 覆盖
  // 或者可以渲染 Sidebar 结构但不显示内容
  
  return (
    <>
      <AlertDialog open={showRepairDialog}>
        <AlertDialogContent>
            <AlertDialogHeader>
                <AlertDialogTitle>配置文件结构损坏</AlertDialogTitle>
                <AlertDialogDescription>
                    检测到配置文件已初始化，但文件结构已损坏，无法正常读取。
                    点击“修复”将重置配置文件结构（保留初始化状态），或请手动检查配置文件。
                </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogAction onClick={handleRepair}>修复配置文件</AlertDialogAction>
            </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {isReady && (
        <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
            <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
            <div className="flex items-center gap-2 px-4">
                <SidebarTrigger className="-ml-1" />
                <Separator
                orientation="vertical"
                className="mr-2 data-[orientation=vertical]:h-4"
                />
                <Breadcrumb>
                <BreadcrumbList>
                    <BreadcrumbItem className="hidden md:block">
                    <BreadcrumbLink href="#">
                        Building Your Application
                    </BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator className="hidden md:block" />
                    <BreadcrumbItem>
                    <BreadcrumbPage>Data Fetching</BreadcrumbPage>
                    </BreadcrumbItem>
                </BreadcrumbList>
                </Breadcrumb>
            </div>
            </header>
            <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
            <div className="grid auto-rows-min gap-4 md:grid-cols-3">
                <div className="bg-muted/50 aspect-video rounded-xl" />
                <div className="bg-muted/50 aspect-video rounded-xl" />
                <div className="bg-muted/50 aspect-video rounded-xl" />
            </div>
            <div className="bg-muted/50 min-h-[100vh] flex-1 rounded-xl md:min-h-min" />
            </div>
        </SidebarInset>
        </SidebarProvider>
      )}
    </>
  )
}
