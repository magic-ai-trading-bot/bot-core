import * as React from "react"
import * as SliderPrimitive from "@radix-ui/react-slider"

import { cn } from "@/lib/utils"

/**
 * Premium Slider Component
 * Uses cyan/emerald gradient to match luxury design system
 * Consistent with PremiumSwitch and other luxury components
 */
const Slider = React.forwardRef<
  React.ElementRef<typeof SliderPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof SliderPrimitive.Root>
>(({ className, ...props }, ref) => (
  <SliderPrimitive.Root
    ref={ref}
    className={cn(
      "relative flex w-full touch-none select-none items-center",
      className
    )}
    {...props}
  >
    <SliderPrimitive.Track
      className={cn(
        "relative h-2 w-full grow overflow-hidden rounded-full",
        // Light mode: subtle gray background
        "bg-black/10 dark:bg-white/10"
      )}
    >
      <SliderPrimitive.Range
        className={cn(
          "absolute h-full",
          // Gradient matching PremiumSwitch: cyan to emerald
          "bg-gradient-to-r from-[#00D9FF] to-[#22c55e]"
        )}
      />
    </SliderPrimitive.Track>
    <SliderPrimitive.Thumb
      className={cn(
        "block h-5 w-5 rounded-full shadow-lg",
        // Background and border
        "bg-white dark:bg-white border-2",
        "border-[#00D9FF]",
        // Glow effect on focus
        "ring-offset-background transition-all duration-200",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00D9FF]/50 focus-visible:ring-offset-2",
        // Hover glow
        "hover:shadow-[0_0_10px_rgba(0,217,255,0.5)]",
        "disabled:pointer-events-none disabled:opacity-50"
      )}
    />
  </SliderPrimitive.Root>
))
Slider.displayName = SliderPrimitive.Root.displayName

export { Slider }
