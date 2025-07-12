import { cva, VariantProps } from "class-variance-authority";
import React from "react";
import { cn } from "../../utils/utils";

const OpacityStyles = cva("bg-white", {
  variants: {
    level: {
      "extra-light": "bg-opacity-10",
      light: "bg-opacity-25",
      medium: "bg-opacity-50",
      "semi-dark": "bg-opacity-75",
      full: "bg-opacity-100",
    },
  },
  defaultVariants: {
    level: "full",
  },
});

export type OpacityProps = React.HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof OpacityStyles> & {
    children: React.ReactNode;
  };

export const Opacity: React.FC<OpacityProps> = ({
  level,
  className,
  children,
  ...props
}) => {
  return (
    <div {...props} className={cn(OpacityStyles({ level }), className)}>
      {children}
    </div>
  );
};
