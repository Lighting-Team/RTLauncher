"use client"

// 导航图标
import {
  Home,
  Download,
  Wrench,
  Settings,
} from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from "@/components/ui/avatar"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip"

interface SidebarProps {
  className?: string
}

interface NavItem {
  icon: React.ReactNode
  label: string
  href?: string
  isAvatar?: boolean
}

// 顶部导航项
const topNavItems: NavItem[] = [
  { icon: <Home className="size-4" />, label: "首页", href: "/" },
  { icon: <Download className="size-4" />, label: "下载", href: "/downloads" },
  { icon: <Wrench className="size-4" />, label: "工具", href: "/tools" },
]

// 底部导航项
const bottomNavItems: NavItem[] = [
  { icon: <Settings className="size-4" />, label: "设置", href: "/settings" },
  {
    icon: (
      <Avatar size="sm">
        <AvatarImage src="https://github.com/shadcn.png" alt="User" />
        <AvatarFallback>U</AvatarFallback>
      </Avatar>
    ),
    label: "个人",
    href: "/profile",
    isAvatar: true,
  },
]

// 导航按钮
function NavButton({ item }: { item: NavItem }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        {item.isAvatar ? (
          <button
            type="button"
            className="flex size-9 items-center justify-center rounded-4xl"
          >
            {item.icon}
          </button>
        ) : (
          <Button variant="ghost" size="icon">
            {item.icon}
          </Button>
        )}
      </TooltipTrigger>
      <TooltipContent side="right">
        <p>{item.label}</p>
      </TooltipContent>
    </Tooltip>
  )
}

// 左侧边栏
export function Sidebar({ className }: SidebarProps) {
  return (
    <aside
      className={cn(
        "flex h-full w-14 flex-col border-r border-border bg-sidebar",
        className
      )}
    >
      <nav className="flex flex-1 flex-col items-center gap-2 p-2">
        {topNavItems.map((item, index) => (
          <NavButton key={index} item={item} />
        ))}
      </nav>

      <div className="flex flex-col items-center gap-2 border-t border-border p-2">
        {bottomNavItems.map((item, index) => (
          <NavButton key={index} item={item} />
        ))}
      </div>
    </aside>
  )
}
