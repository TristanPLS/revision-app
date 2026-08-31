import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        // active: = retour visuel au tap (hover: est sous @media (hover:hover))
        default: "bg-primary text-primary-foreground hover:opacity-90 active:opacity-80",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-secondary/80 active:opacity-80",
        outline:
          "border border-input bg-background hover:bg-accent hover:text-accent-foreground active:bg-accent",
        ghost: "hover:bg-accent hover:text-accent-foreground active:bg-accent active:text-accent-foreground",
        destructive:
          "bg-destructive text-destructive-foreground hover:opacity-90 active:opacity-80",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        // 44px min height on touch = comfortable target (phone use over Tailscale);
        // sm redescend à 36px à partir du breakpoint sm.
        default: "h-11 px-5 py-2",
        sm: "h-11 px-4 sm:h-9 sm:px-3 rounded-md text-sm",
        lg: "h-12 rounded-md px-7 text-base",
        icon: "h-11 w-11",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => (
    <button
      ref={ref}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  )
);
Button.displayName = "Button";

export { Button, buttonVariants };
