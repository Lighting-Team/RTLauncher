"use client"

import { useEffect } from "react"
import { useRouter } from "next/navigation"
import { invoke } from "@tauri-apps/api/core"

export default function Page() {
  const router = useRouter()

  useEffect(() => {
    async function checkInit() {
      try {
        const isInitialized = await invoke<boolean>("check_initialization")
        if (!isInitialized) {
          router.push("/initialize")
        }
        // If initialized, stay on this page (which is currently null/blank)
      } catch (error) {
        console.error("Failed to check initialization status:", error)
        // In case of error, you might want to default to one state or show an error
        // For safety, let's assume not initialized or retry
        // But per requirements, only explicit logic:
        // "If no config ... then enter Initialize page"
      }
    }

    checkInit()
  }, [router])

  return null
}
