import { cva, VariantProps } from "class-variance-authority";
import React, { useEffect, useState } from "react";
import { Opacity, OpacityProps } from "./Opacity";

const modalStyles = cva(
  [
    "fixed inset-0 z-50 flex items-center justify-center  transition-all bg-dark-bg bg-opacity-50 ",
  ],
  {
    variants: {
      open: {
        true: "bg-black",
        false: "opacity-0 pointer-events-none",
      },
    },
    defaultVariants: {
      open: false,
    },
  }
);

type ModalProps = VariantProps<typeof modalStyles> & {
  onClose?: () => void;
  children: React.ReactNode;
};

const Modal: React.FC<ModalProps> & {
  Children: React.FC<ChildrenProps>;
} = ({ open, onClose, children }) => {
  //INFO: This is a workaround to prevent the modal from animating out when it first mounts
  const [hasOpened, setHasOpened] = useState(false);

  useEffect(() => {
    if (open) {
      setHasOpened(true);
    }
  }, [open]);

  const handleOverlayClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (onClose) {
      e.stopPropagation();
      onClose();
    }
  };

  return (
    <div
      className={`w-full backdrop-blur-[6px] ${modalStyles({ open })} ${
        hasOpened ? (open ? "animate-fade-in" : "animate-fade-out") : ""
      }`}
      onClick={handleOverlayClick}
    >
      <div
        className={`mx-2 transform transition-transform ${
          open ? "animate-scale-in" : "animate-scale-out"
        }`}
      >
        {children}
      </div>
    </div>
  );
};

type ChildrenProps = {
  children: React.ReactNode;
  opacityLevel: OpacityProps["level"];
  className?: string;
};

const Children: React.FC<ChildrenProps> = ({
  children,
  className,
  opacityLevel,
  ...props
}) => {
  const handleStopPropagation = (e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
  };

  return (
    <Opacity
      level={opacityLevel}
      onClick={handleStopPropagation}
      className={`mx-auto w-full ${className ? className : ""}`}
      {...props}
    >
      {children}
    </Opacity>
  );
};

Modal.Children = Children;

export { Modal };
