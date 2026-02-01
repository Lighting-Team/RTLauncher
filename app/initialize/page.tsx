"use client"

import * as React from "react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { ChevronRight, CheckCircle2, ChevronLeft, Loader2, FileWarning, Search, Sun, Moon, Monitor } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"
import { useTheme } from "next-themes"
import { ModeToggle } from "@/components/theme-toggle"
import { useRouter } from "next/navigation"


export default function StartPage() {
  const router = useRouter()
  const [step, setStep] = React.useState(1)
  const [isConfigCreated, setIsConfigCreated] = React.useState(false)
  const totalSteps = 4


  const nextStep = () => setStep((prev) => Math.min(prev + 1, totalSteps))
  const prevStep = () => setStep((prev) => Math.max(prev - 1, 1))

  const handleConfigCreated = () => {
    setIsConfigCreated(true)
  }

  const handleConfigInvalidated = () => {
    setIsConfigCreated(false)
  }

  const handleFinish = async () => {
    try {
      await invoke("complete_initialization")
      router.push('/')
    } catch (err) {
      console.error("Failed to complete initialization:", err)
      // 可以选择添加一个 toast 提示，或者这里暂且静默失败，因为影响不大
      router.push('/')
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30 p-4">
      <Card className="w-full max-w-lg shadow-xl border-muted">
        <CardHeader className="space-y-4">
          {/* Progress Indicator */}
          <div className="flex items-center justify-between">
            <div className="flex space-x-1.5">
              {Array.from({ length: totalSteps }).map((_, i) => (
                <div
                  key={i}
                  className={`h-1.5 w-8 rounded-full ${
                    i + 1 <= step ? "bg-primary" : "bg-muted"
                  }`}
                />
              ))}
            </div>
            <span className="text-xs font-medium text-muted-foreground">
              Step {step} of {totalSteps}
            </span>
          </div>

          <div className="flex items-start justify-between gap-4">
            <div className="space-y-1.5">
              <CardTitle className="text-2xl font-bold tracking-tight">
                {step === 1 && "欢迎使用 RTLauncher"}
                {step === 2 && "外观设置"}
                {step === 3 && "基本设置"}
                {step === 4 && "准备就绪"}
              </CardTitle>
              <CardDescription className="text-base">
                {step === 1 && "看起来你是第一次使用 RTLauncher，让我们开始配置您的启动器环境。"}
                {step === 2 && "选择您喜欢的主题模式。"}
                {step === 3 && "初始化配置文件。"}
                {step === 4 && "配置已完成，您可以开始使用了。"}
              </CardDescription>
            </div>
            {step > 2 && (
              <div className="shrink-0">
                <ModeToggle />
              </div>
            )}
          </div>
        </CardHeader>

        <CardContent className="min-h-[200px] py-2">
          <div>
            {step === 1 && <StepOne />}
            {step === 2 && <StepTheme />}
            {step === 3 && (
              <StepTwo 
                onConfigCreated={handleConfigCreated} 
                onConfigInvalidated={handleConfigInvalidated} 
                isConfigCreated={isConfigCreated} 
              />
            )}
            {step === 4 && <StepThree />}
          </div>
        </CardContent>

        <CardFooter className="flex justify-between pt-6">
          <Button
            variant="ghost"
            onClick={prevStep}
            disabled={step === 1}
          >
            {step !== 1 && <ChevronLeft className="mr-2 h-4 w-4" />}
            {step !== 1 ? "上一步" : ""}
          </Button>
          <Button 
            onClick={step === totalSteps ? handleFinish : nextStep}
            disabled={step === 3 && !isConfigCreated}
          >
            {step === totalSteps ? "完成" : "下一步"}
            {step !== totalSteps && <ChevronRight className="ml-2 h-4 w-4" />}
          </Button>
        </CardFooter>
      </Card>
    </div>
  )
}

function StepOne() {
  return (
    <div className="space-y-6">
      <div className="grid gap-3">
        <Label htmlFor="language" className="text-base">选择语言</Label>
        <Select defaultValue="zh-CN">
          <SelectTrigger id="language" className="h-11">
            <SelectValue placeholder="Select language" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="zh-CN">简体中文</SelectItem>
          </SelectContent>
        </Select>
        <p className="text-[0.8rem] text-muted-foreground">
          您稍后可以在设置中更改此选项。
        </p>
      </div>
    </div>
  )
}

function ThemePreviewLight() {
  return (
    <div className="space-y-2 rounded-lg bg-[#ecedef] p-2">
      <div className="space-y-2 rounded-md bg-white p-2 shadow-sm">
        <div className="h-2 w-[40px] rounded-lg bg-[#ecedef]" />
        <div className="h-2 w-[60px] rounded-lg bg-[#ecedef]" />
      </div>
      <div className="flex items-center space-x-2 rounded-md bg-white p-2 shadow-sm">
        <div className="h-4 w-4 rounded-full bg-[#ecedef]" />
        <div className="h-2 w-[60px] rounded-lg bg-[#ecedef]" />
      </div>
    </div>
  )
}

function ThemePreviewDark() {
  return (
    <div className="space-y-2 rounded-lg bg-slate-950 p-2">
      <div className="space-y-2 rounded-md bg-slate-800 p-2 shadow-sm">
        <div className="h-2 w-[40px] rounded-lg bg-slate-400" />
        <div className="h-2 w-[60px] rounded-lg bg-slate-400" />
      </div>
      <div className="flex items-center space-x-2 rounded-md bg-slate-800 p-2 shadow-sm">
        <div className="h-4 w-4 rounded-full bg-slate-400" />
        <div className="h-2 w-[60px] rounded-lg bg-slate-400" />
      </div>
    </div>
  )
}

function StepTheme() {
  const { setTheme, theme, systemTheme } = useTheme()
  const [mounted, setMounted] = React.useState(false)

  React.useEffect(() => {
    setMounted(true)
  }, [])

  if (!mounted) {
    return <div className="h-[200px]" />
  }

  return (
    <div className="grid grid-cols-3 gap-4">
      <div 
        className={`cursor-pointer rounded-xl border-2 p-1 hover:bg-accent hover:text-accent-foreground ${theme === 'light' ? 'border-primary ring-2 ring-primary/20' : 'border-muted'}`}
        onClick={() => setTheme("light")}
      >
        <ThemePreviewLight />
        <div className="flex items-center justify-center p-2 font-medium text-sm gap-2">
           <Sun className="h-4 w-4" />
           <span className="hidden sm:inline">Light</span>
        </div>
      </div>
      
      <div 
        className={`cursor-pointer rounded-xl border-2 p-1 hover:bg-accent hover:text-accent-foreground ${theme === 'dark' ? 'border-primary ring-2 ring-primary/20' : 'border-muted'}`}
        onClick={() => setTheme("dark")}
      >
        <ThemePreviewDark />
        <div className="flex items-center justify-center p-2 font-medium text-sm gap-2">
           <Moon className="h-4 w-4" />
           <span className="hidden sm:inline">Dark</span>
        </div>
      </div>

      <div 
        className={`cursor-pointer rounded-xl border-2 p-1 hover:bg-accent hover:text-accent-foreground ${theme === 'system' ? 'border-primary ring-2 ring-primary/20' : 'border-muted'}`}
        onClick={() => setTheme("system")}
      >
        {systemTheme === 'dark' ? <ThemePreviewDark /> : <ThemePreviewLight />}
        <div className="flex items-center justify-center p-2 font-medium text-sm gap-2">
           <Monitor className="h-4 w-4" />
           <span className="hidden sm:inline">System</span>
        </div>
      </div>
    </div>
  )
}

function StepTwo({ 
  onConfigCreated, 
  onConfigInvalidated, 
  isConfigCreated 
}: { 
  onConfigCreated: () => void, 
  onConfigInvalidated: () => void, 
  isConfigCreated: boolean 
}) {
  type ConfigStatus = "ok" | "missing" | "invalid_json" | "invalid_data" | "read_error"
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

  const [loading, setLoading] = React.useState(false)
  const [checking, setChecking] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)
  const [configPath, setConfigPath] = React.useState<string>("")
  const [configStatus, setConfigStatus] = React.useState<ConfigStatus>("missing")
  const [configInfo, setConfigInfo] = React.useState<AppConfig["launcher"] | null>(null)

  const refreshStatus = React.useCallback(async (withDelay = true) => {
    setChecking(true)
    setError(null)
    try {
      const result = await invoke<ConfigCheckResult>("check_config_status")
      setConfigStatus(result.status)
      setConfigInfo(result.config?.launcher ?? null)
      if (result.error) {
        setError(result.error)
      }
      if (result.status === "ok") {
        onConfigCreated()
      } else {
        // 如果状态不是 ok，但 isConfigCreated 为 true，说明配置已失效，需要通知父组件
        if (isConfigCreated) {
          onConfigInvalidated()
        }
      }
    } catch (err) {
      console.error(err)
      setConfigStatus("read_error")
      setConfigInfo(null)
      setError("检测配置文件失败，请重试。")
      if (isConfigCreated) {
        onConfigInvalidated()
      }
    } finally {
      if (withDelay) {
        setTimeout(() => setChecking(false), 800)
      } else {
        setChecking(false)
      }
    }
  }, [onConfigCreated, onConfigInvalidated, isConfigCreated])

  const checkAndCreateConfig = React.useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      await invoke("create_config_files")
      await refreshStatus(false)
    } catch (err) {
      console.error(err)
      setError("初始化配置文件失败，请重试。")
    } finally {
      setLoading(false)
    }
  }, [refreshStatus])

  // Initial check on mount
  React.useEffect(() => {
      // Fetch config path
      invoke<string>("get_config_path").then(setConfigPath).catch(console.error)

      // 每次挂载都重新检测，确保文件状态最新
      refreshStatus()
  }, [refreshStatus])


  if (checking) {
      return (
        <div className="flex flex-col items-center justify-center space-y-6 py-6 text-center">
            <div className="rounded-full bg-muted p-3">
                <Search className="h-8 w-8 text-muted-foreground animate-pulse" />
            </div>
            <div className="space-y-2">
                <h3 className="text-lg font-medium">正在检测环境...</h3>
                <p className="text-sm text-muted-foreground">
                    正在扫描本地配置文件，请稍候。
                </p>
            </div>
        </div>
      )
  }

  if (isConfigCreated) {
      const generatedAtLabel = configInfo?.generated_at
        ? new Date(configInfo.generated_at * 1000).toLocaleString()
        : "-"
      return (
        <div className="flex flex-col items-center justify-center space-y-6 py-6 text-center">
            <div className="rounded-full bg-green-100 p-3 dark:bg-green-900/20">
                <CheckCircle2 className="h-8 w-8 text-green-600 dark:text-green-500" />
            </div>
            <div className="space-y-2">
                <h3 className="text-lg font-medium">配置文件就绪</h3>
                <p className="text-sm text-muted-foreground">
                    初始化检查通过，您可以继续下一步。
                </p>
                {configInfo && (
                  <div className="mt-4 w-full rounded-md bg-muted p-3 text-xs text-muted-foreground">
                    <div className="flex items-center justify-between">
                      <span>版本</span>
                      <span className="font-mono">{configInfo.version || "-"}</span>
                    </div>
                    <div className="mt-2 flex items-center justify-between">
                      <span>生成时间</span>
                      <span className="font-mono">{generatedAtLabel}</span>
                    </div>
                    <div className="mt-2 flex items-center justify-between">
                      <span>初始化状态</span>
                      <span className="font-mono">{configInfo.initialized ? "true" : "false"}</span>
                    </div>
                  </div>
                )}
                {configPath && (
                    <div className="mt-4 rounded-md bg-muted p-2 text-xs font-mono text-muted-foreground break-all">
                        {configPath}
                    </div>
                )}
            </div>
        </div>
      )
  }

  const statusTitle = (() => {
    switch (configStatus) {
      case "missing":
        return "未找到配置文件"
      case "invalid_json":
        return "配置文件已损坏"
      case "invalid_data":
        return "配置内容不完整"
      case "read_error":
        return "无法读取配置文件"
      default:
        return "配置文件异常"
    }
  })()

  const statusDescription = (() => {
    switch (configStatus) {
      case "missing":
        return "我们需要在您的设备上创建必要的配置文件和目录结构以运行启动器。"
      case "invalid_json":
        return "配置文件格式不正确，需要重新生成以继续。"
      case "invalid_data":
        return "配置文件存在，但关键字段缺失或不合法。"
      case "read_error":
        return "读取配置文件失败，请检查文件权限或磁盘状态。"
      default:
        return "配置文件状态异常，请重新初始化。"
    }
  })()

  const actionLabel = configStatus === "missing" ? "开始初始化" : "重新初始化"

  return (
    <div className="space-y-6">
      <div className="grid gap-3">
        <div className="rounded-md border border-dashed p-6 flex flex-col items-center justify-center text-center gap-4 bg-muted/20">
            <div className="p-3 bg-muted rounded-full">
                <FileWarning className="h-6 w-6 text-muted-foreground" />
            </div>
            <div className="space-y-1">
                <p className="text-sm font-medium">{statusTitle}</p>
                <p className="text-xs text-muted-foreground max-w-[260px] mx-auto">
                    {statusDescription}
                </p>
            </div>

            {configInfo && (
              <div className="w-full rounded-md bg-muted/50 p-2 text-xs text-muted-foreground">
                <div className="flex items-center justify-between">
                  <span>版本</span>
                  <span className="font-mono">{configInfo.version || "-"}</span>
                </div>
                <div className="mt-2 flex items-center justify-between">
                  <span>生成时间</span>
                  <span className="font-mono">
                    {configInfo.generated_at ? new Date(configInfo.generated_at * 1000).toLocaleString() : "-"}
                  </span>
                </div>
                <div className="mt-2 flex items-center justify-between">
                  <span>初始化状态</span>
                  <span className="font-mono">{configInfo.initialized ? "true" : "false"}</span>
                </div>
              </div>
            )}

            {configPath && (
                <div className="w-full rounded-md bg-muted/50 p-2 text-xs font-mono text-muted-foreground break-all border border-muted">
                    {configPath}
                </div>
            )}

            {error && (
                <p className="text-xs text-red-500 font-medium break-all">{error}</p>
            )}

            <Button onClick={checkAndCreateConfig} disabled={loading}>
                {loading ? (
                    <>
                        <Loader2 className="mr-2 h-4 w-4" />
                        正在初始化...
                    </>
                ) : (
                    actionLabel
                )}
            </Button>
        </div>
      </div>
    </div>
  )
}

function StepThree() {
  return (
    <div className="flex flex-col items-center justify-center space-y-6 py-6 text-center">
            <div className="rounded-full bg-green-100 p-3 dark:bg-green-900/20">
                <CheckCircle2 className="h-8 w-8 text-green-600 dark:text-green-500" />
            </div>
      <div className="space-y-2 max-w-xs mx-auto">
        <h3 className="text-lg font-semibold text-foreground">初始化完成</h3>
        <p className="text-sm text-muted-foreground">
          RTLauncher 已准备就绪。点击下方按钮进入主界面，开始您的游戏之旅。
        </p>
      </div>
    </div>
  )
}
