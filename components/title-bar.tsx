"use client"

import { useState, useEffect, useCallback } from "react"
import { Button } from "@/components/ui/button"
import { Minus, Copy, X, Maximize2 } from "lucide-react"
import { cn } from "@/lib/utils"

interface TitleBarProps {
  className?: string
}

export function TitleBar({ className }: TitleBarProps) {
  const [isMaximized, setIsMaximized] = useState(false)
  const [windowApi, setWindowApi] = useState<{
    minimize: () => Promise<void>
    maximize: () => Promise<void>
    unmaximize: () => Promise<void>
    toggleMaximize: () => Promise<void>
    close: () => Promise<void>
    isMaximized: () => Promise<boolean>
    onResized: (handler: () => void) => Promise<() => void>
  } | null>(null)

  useEffect(() => {
    let unlisten: (() => void) | undefined

    const initWindow = async () => {
      try {
        if (typeof window !== "undefined") {
          // Use getCurrentWebviewWindow for Tauri 2.0
          const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow")
          const webviewWindow = getCurrentWebviewWindow()
          
          setWindowApi({
            minimize: () => webviewWindow.minimize(),
            maximize: () => webviewWindow.maximize(),
            unmaximize: () => webviewWindow.unmaximize(),
            toggleMaximize: () => webviewWindow.toggleMaximize(),
            close: () => webviewWindow.close(),
            isMaximized: () => webviewWindow.isMaximized(),
            onResized: (handler) => webviewWindow.onResized(handler),
          })

          // Check initial maximize state
          const maximized = await webviewWindow.isMaximized()
          setIsMaximized(maximized)

          // Listen for resize events
          unlisten = await webviewWindow.onResized(() => {
            webviewWindow.isMaximized().then(setIsMaximized)
          })
        }
      } catch (err) {
        console.log("Window API not available:", err)
      }
    }

    initWindow()

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  const handleMinimize = useCallback(async () => {
    if (windowApi) {
      try {
        await windowApi.minimize()
      } catch (err) {
        console.error("Minimize failed:", err)
      }
    }
  }, [windowApi])

  const handleMaximizeRestore = useCallback(async () => {
    if (windowApi) {
      try {
        if (isMaximized) {
          await windowApi.unmaximize()
        } else {
          await windowApi.maximize()
        }
      } catch (err) {
        console.error("Maximize/Restore failed:", err)
      }
    }
  }, [windowApi, isMaximized])

  const handleClose = useCallback(async () => {
    if (windowApi) {
      try {
        await windowApi.close()
      } catch (err) {
        console.error("Close failed:", err)
      }
    }
  }, [windowApi])

  return (
    <div
      className={cn(
        "h-10 bg-background/95 backdrop-blur border-b border-border flex items-center select-none",
        className
      )}
      data-tauri-drag-region
    >
      {/* Left: App Icon and Title - Draggable */}
      <div 
        className="flex items-center gap-2 px-3 h-full"
        data-tauri-drag-region
      >
        <div 
          className="w-5 h-5 bg-contain bg-no-repeat bg-center"
          style={{ backgroundImage: "url('/rtlauncher.svg')" }}
          data-tauri-drag-region
        />
        <span 
          className="text-sm font-medium text-foreground/80"
          data-tauri-drag-region
        >
          RTLauncher
        </span>
      </div>

      {/* Center: Draggable area */}
      <div className="flex-1 h-full" data-tauri-drag-region />

      {/* Right: Window Controls - NOT draggable */}
      <div className="flex items-center h-full no-drag">
        <WindowButton
          onClick={handleMinimize}
          title="最小化"
        >
          <Minus className="size-4" />
        </WindowButton>
        
        <WindowButton
          onClick={handleMaximizeRestore}
          title={isMaximized ? "还原" : "最大化"}
        >
          {isMaximized ? (
            <Copy className="size-3.5 rotate-90" />
          ) : (
            <Maximize2 className="size-3.5" />
          )}
        </WindowButton>
        
        <WindowButton
          onClick={handleClose}
          title="关闭"
          isClose
        >
          <X className="size-4" />
        </WindowButton>
      </div>
    </div>
  )
}

// Window Control Button Component
interface WindowButtonProps {
  onClick: () => void
  title: string
  children: React.ReactNode
  isClose?: boolean
}

function WindowButton({ onClick, title, children, isClose }: WindowButtonProps) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      className={cn(
        "h-full w-11 rounded-none border-0",
        "focus-visible:ring-0 focus-visible:ring-offset-0",
        "transition-colors",
        isClose 
          ? "hover:bg-destructive hover:text-destructive-foreground active:bg-destructive" 
          : "hover:bg-muted/80 active:bg-muted"
      )}
      onClick={onClick}
      title={title}
    >
      {children}
    </Button>
  )
}
