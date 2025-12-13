import * as React from "react"
import * as SwitchPrimitives from "@radix-ui/react-switch"

import { cn } from "@/lib/utils"

/**
 * Premium Switch Component
 * Uses cyan/emerald gradient to match luxury design system
 * Consistent with Slider and other luxury components
 */
const Switch = React.forwardRef<
  React.ElementRef<typeof SwitchPrimitives.Root>,
  React.ComponentPropsWithoutRef<typeof SwitchPrimitives.Root>
>(({ className, ...props }, ref) => (
  <SwitchPrimitives.Root
    className={cn(
      // Base styles - fixed dimensions that won't be affected by parent
      "peer inline-flex shrink-0 cursor-pointer items-center rounded-full",
      // Fixed sizing with !important to override any conflicts
      "!h-6 !w-11",
      // Border and states
      "border-2 border-transparent",
      "transition-all duration-300",
      // Focus styles
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00D9FF]/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      // Disabled state
      "disabled:cursor-not-allowed disabled:opacity-50",
      // Unchecked: subtle gray background
      "data-[state=unchecked]:bg-black/10 dark:data-[state=unchecked]:bg-white/10",
      // Checked: cyan to emerald gradient (matching Slider)
      "data-[state=checked]:bg-gradient-to-r data-[state=checked]:from-[#00D9FF] data-[state=checked]:to-[#22c55e]",
      // Glow effect when checked
      "data-[state=checked]:shadow-[0_0_10px_rgba(0,217,255,0.4)]",
      className
    )}
    style={{ height: '24px', width: '44px', minHeight: '24px', maxHeight: '24px' }}
    {...props}
    ref={ref}
  >
    <SwitchPrimitives.Thumb
      className={cn(
        // Base styles
        "pointer-events-none block rounded-full bg-white shadow-lg ring-0",
        // Fixed thumb sizing with !important
        "!h-5 !w-5",
        // Transition for smooth sliding
        "transition-transform duration-300",
        // Position based on state
        "data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0"
      )}
      style={{ height: '20px', width: '20px', minHeight: '20px', maxHeight: '20px' }}
    />
  </SwitchPrimitives.Root>
))
Switch.displayName = SwitchPrimitives.Root.displayName

export { Switch }
