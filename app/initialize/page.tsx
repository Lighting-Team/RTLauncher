"use client"

import * as React from "react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { ChevronRight, CheckCircle2, ChevronLeft, Loader2, FileWarning, Search } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"
import { ModeToggle } from "@/components/theme-toggle"
import { useRouter } from "next/navigation"

export default function StartPage() {
  const [step, setStep] = React.useState(1)
  const [isConfigCreated, setIsConfigCreated] = React.useState(false)
  const totalSteps = 4
  const router = useRouter()

  const nextStep = () => setStep((prev) => Math.min(prev + 1, totalSteps))
  const prevStep = () => setStep((prev) => Math.max(prev - 1, 1))

  const handleConfigCreated = () => {
    setIsConfigCreated(true)
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
            {step === 3 && <StepTwo onConfigCreated={handleConfigCreated} isConfigCreated={isConfigCreated} />}
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
            onClick={step === totalSteps ? () => router.push("/") : nextStep}
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

function StepTheme() {
  return (
    <div className="flex items-center justify-center py-10">
      <ModeToggle />
    </div>
  )
}

function StepTwo({ onConfigCreated, isConfigCreated }: { onConfigCreated: () => void, isConfigCreated: boolean }) {
  const [loading, setLoading] = React.useState(false)
  const [checking, setChecking] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)
  const [configPath, setConfigPath] = React.useState<string>("")

  const checkAndCreateConfig = React.useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const exists = await invoke<boolean>("check_config_files")
      if (exists) {
        onConfigCreated()
      } else {
        await invoke("create_config_files")
        onConfigCreated()
      }
    } catch (err) {
        console.error(err)
      setError("初始化配置文件失败，请重试。")
    } finally {
      setLoading(false)
    }
  }, [onConfigCreated])

  // Initial check on mount
  React.useEffect(() => {
      // Fetch config path
      invoke<string>("get_config_path").then(setConfigPath).catch(console.error)

      if (!isConfigCreated) {
           setChecking(true)
           invoke<boolean>("check_config_files").then(exists => {
               if (exists) {
                   onConfigCreated()
               }
           }).catch(console.error)
           .finally(() => {
               // Add a small delay for better UX so the checking state isn't just a flicker
               setTimeout(() => setChecking(false), 800)
           })
      } else {
        setChecking(false)
      }
  }, [isConfigCreated, onConfigCreated])


  if (isConfigCreated) {
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
                {configPath && (
                    <div className="mt-4 rounded-md bg-muted p-2 text-xs font-mono text-muted-foreground break-all">
                        {configPath}
                    </div>
                )}
            </div>
        </div>
      )
  }

  if (checking) {
      return (
        <div className="flex flex-col items-center justify-center space-y-6 py-6 text-center">
            <div className="rounded-full bg-muted p-3">
                <Search className="h-8 w-8 text-muted-foreground" />
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

  return (
    <div className="space-y-6">
      <div className="grid gap-3">
        <div className="rounded-md border border-dashed p-6 flex flex-col items-center justify-center text-center gap-4 bg-muted/20">
            <div className="p-3 bg-muted rounded-full">
                <FileWarning className="h-6 w-6 text-muted-foreground" />
            </div>
            <div className="space-y-1">
                <p className="text-sm font-medium">检测到缺少配置文件</p>
                <p className="text-xs text-muted-foreground max-w-[260px] mx-auto">
                    我们需要在您的设备上创建必要的配置文件和目录结构以运行启动器。
                </p>
            </div>
            
            {configPath && (
                <div className="w-full rounded-md bg-muted/50 p-2 text-xs font-mono text-muted-foreground break-all border border-muted">
                    {configPath}
                </div>
            )}

            {error && (
                <p className="text-sm text-red-500 font-medium">{error}</p>
            )}

            <Button onClick={checkAndCreateConfig} disabled={loading}>
                {loading ? (
                    <>
                        <Loader2 className="mr-2 h-4 w-4" />
                        正在初始化...
                    </>
                ) : (
                    "开始初始化"
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
