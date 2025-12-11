import * as React from "react"
import * as SwitchPrimitives from "@radix-ui/react-switch"

import { cn } from "@/lib/utils"

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
      "transition-colors duration-200",
      // Focus styles
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      // Disabled state
      "disabled:cursor-not-allowed disabled:opacity-50",
      // Checked/unchecked background colors
      "data-[state=checked]:bg-primary data-[state=unchecked]:bg-input",
      className
    )}
    style={{ height: '24px', width: '44px', minHeight: '24px', maxHeight: '24px' }}
    {...props}
    ref={ref}
  >
    <SwitchPrimitives.Thumb
      className={cn(
        // Base styles
        "pointer-events-none block rounded-full bg-background shadow-lg ring-0",
        // Fixed thumb sizing with !important
        "!h-5 !w-5",
        // Transition for smooth sliding
        "transition-transform duration-200",
        // Position based on state
        "data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0"
      )}
      style={{ height: '20px', width: '20px', minHeight: '20px', maxHeight: '20px' }}
    />
  </SwitchPrimitives.Root>
))
Switch.displayName = SwitchPrimitives.Root.displayName

export { Switch }
