import * as React from "react"
import * as CheckboxPrimitive from "@radix-ui/react-checkbox"
import { Check } from "lucide-react"

import { cn } from "@/lib/utils"

/**
 * Premium Checkbox Component
 * Uses cyan/emerald gradient to match luxury design system
 */
const Checkbox = React.forwardRef<
  React.ElementRef<typeof CheckboxPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof CheckboxPrimitive.Root>
>(({ className, ...props }, ref) => (
  <CheckboxPrimitive.Root
    ref={ref}
    className={cn(
      "peer h-4 w-4 shrink-0 rounded-sm",
      // Border - cyan when unchecked
      "border border-[#00D9FF]/50",
      // Focus styles
      "ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00D9FF]/50 focus-visible:ring-offset-2",
      // Disabled
      "disabled:cursor-not-allowed disabled:opacity-50",
      // Checked state - cyan/emerald gradient
      "data-[state=checked]:bg-gradient-to-r data-[state=checked]:from-[#00D9FF] data-[state=checked]:to-[#22c55e]",
      "data-[state=checked]:border-transparent data-[state=checked]:text-white",
      "transition-all duration-200",
      className
    )}
    {...props}
  >
    <CheckboxPrimitive.Indicator
      className={cn("flex items-center justify-center text-current")}
    >
      <Check className="h-4 w-4" />
    </CheckboxPrimitive.Indicator>
  </CheckboxPrimitive.Root>
))
Checkbox.displayName = CheckboxPrimitive.Root.displayName

export { Checkbox }
